//! ACP 客户端后台 actor 运行时。
//!
//! 该模块把代理进程、ACP 连接和会话请求串行化到独立线程中，避免上层
//! `AcpClient` 在多个异步调用之间直接共享非线程安全的 ACP 连接状态。

use super::*;

impl AcpClient {
    /// 在专用线程内启动单线程 Tokio runtime 并运行 actor 循环。
    ///
    /// `startup_tx` 用于把 runtime 初始化结果传回调用方；初始化失败时会清理
    /// actor 状态，避免后续请求误以为后台线程可用。
    pub(super) fn run_actor_thread(
        self,
        command_rx: mpsc::UnboundedReceiver<ActorCommand>,
        startup_tx: oneshot::Sender<Result<(), AcpError>>,
    ) {
        let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(runtime) => runtime,
            Err(err) => {
                let _ = startup_tx.send(Err(AcpError::Initialize(err.to_string())));
                self.invalidate_actor();
                return;
            }
        };

        runtime.block_on(async move {
            let local_set = tokio::task::LocalSet::new();
            local_set
                .run_until(async move {
                    let _ = startup_tx.send(Ok(()));
                    self.actor_loop(command_rx).await;
                })
                .await;
        });
    }

    async fn actor_loop(&self, mut command_rx: mpsc::UnboundedReceiver<ActorCommand>) {
        let mut runtime = None::<ActorRuntime>;
        loop {
            let command = if runtime.is_some() {
                tokio::select! {
                    maybe_command = command_rx.recv() => maybe_command,
                    _ = tokio::time::sleep(self.actor_idle_timeout) => {
                        if let Some(active_runtime) = runtime.take() {
                            self.shutdown_actor_runtime(active_runtime, Some("idle_timeout"), false)
                                .await;
                        } else {
                            self.store_reusable_session(None);
                        }
                        continue;
                    }
                    _ = async {
                        if let Some(active_runtime) = runtime.as_mut() {
                            let _ = (&mut active_runtime.io_closed_rx).await;
                        }
                    } => {
                        if runtime.as_ref().is_some_and(|active_runtime| active_runtime.io_task.is_finished()) {
                            if let Some(active_runtime) = runtime.take() {
                                self.shutdown_actor_runtime(active_runtime, Some("io_closed"), false)
                                    .await;
                            } else {
                                self.store_reusable_session(None);
                            }
                        }
                        continue;
                    }
                }
            } else {
                command_rx.recv().await
            };
            let Some(command) = command else {
                break;
            };
            match command {
                ActorCommand::CreateSession { cwd, response_tx } => {
                    let _ = response_tx.send(self.actor_create_session(&mut runtime, cwd).await);
                }
                ActorCommand::LoadSession { session_id, cwd, response_tx } => {
                    let _ = response_tx
                        .send(self.actor_load_session(&mut runtime, session_id, cwd).await);
                }
                ActorCommand::ResumeSession { session_id, cwd, response_tx } => {
                    let _ = response_tx
                        .send(self.actor_resume_session(&mut runtime, session_id, cwd).await);
                }
                ActorCommand::SetSessionMode { session_id, cwd, mode_id, response_tx } => {
                    let _ = response_tx.send(
                        self.actor_set_session_mode(&mut runtime, session_id, cwd, mode_id).await,
                    );
                }
                ActorCommand::SetSessionConfigOption {
                    session_id,
                    cwd,
                    option_name,
                    value_id,
                    response_tx,
                } => {
                    let _ = response_tx.send(
                        self.actor_set_session_config_option(
                            &mut runtime,
                            session_id,
                            cwd,
                            option_name,
                            value_id,
                        )
                        .await,
                    );
                }
                ActorCommand::SetSessionModel { session_id, cwd, model, response_tx } => {
                    let _ = response_tx.send(
                        self.actor_set_session_model(&mut runtime, session_id, cwd, model).await,
                    );
                }
                ActorCommand::RunPrompt { request, event_tx, response_tx } => {
                    let _ = response_tx
                        .send(self.actor_run_prompt(&mut runtime, request, event_tx).await);
                }
                ActorCommand::Close { response_tx } => {
                    if let Some(active_runtime) = runtime.take() {
                        self.shutdown_actor_runtime(active_runtime, Some("client_close"), false)
                            .await;
                    } else {
                        self.store_reusable_session(None);
                    }
                    let _ = response_tx.send(());
                    return;
                }
            }
        }

        if let Some(active_runtime) = runtime.take() {
            self.shutdown_actor_runtime(active_runtime, Some("actor_channel_closed"), false).await;
        } else {
            self.store_reusable_session(None);
        }
    }

    async fn actor_create_session(
        &self,
        runtime: &mut Option<ActorRuntime>,
        cwd: PathBuf,
    ) -> Result<SessionInfo, AcpError> {
        self.prepare_actor_runtime(runtime, cwd.clone()).await?;
        let mut active_runtime = runtime
            .take()
            .ok_or_else(|| AcpError::Initialize("ACP actor runtime is missing".to_string()))?;
        match self.new_session_id(&active_runtime.conn, &cwd).await {
            Ok(session_id) => {
                *active_runtime.expected_session_id.lock() = Some(session_id.clone());
                tokio::task::yield_now().await;
                if let Some(reason) = self.actor_runtime_restart_reason(&mut active_runtime, &cwd) {
                    self.shutdown_actor_runtime(active_runtime, Some(reason), false).await;
                } else {
                    self.store_reusable_session(Some(session_id.clone()));
                    *runtime = Some(active_runtime);
                }
                Ok(SessionInfo { session_id })
            }
            Err(err) => {
                let finalized = self
                    .shutdown_actor_runtime(active_runtime, Some("create_session_failed"), false)
                    .await;
                Err(enrich_acp_error_with_process_context(err, &finalized))
            }
        }
    }

    async fn actor_load_session(
        &self,
        runtime: &mut Option<ActorRuntime>,
        session_id: String,
        cwd: PathBuf,
    ) -> Result<SessionInfo, AcpError> {
        self.prepare_actor_runtime(runtime, cwd.clone()).await?;
        let runtime = runtime
            .as_mut()
            .ok_or_else(|| AcpError::Initialize("ACP actor runtime is missing".to_string()))?;
        self.load_session_id(&runtime.conn, &cwd, session_id.clone()).await?;
        *runtime.expected_session_id.lock() = Some(session_id.clone());
        self.store_reusable_session(Some(session_id.clone()));
        Ok(SessionInfo { session_id })
    }

    async fn actor_resume_session(
        &self,
        runtime: &mut Option<ActorRuntime>,
        session_id: String,
        cwd: PathBuf,
    ) -> Result<SessionInfo, AcpError> {
        self.prepare_actor_runtime(runtime, cwd.clone()).await?;
        let runtime = runtime
            .as_mut()
            .ok_or_else(|| AcpError::Initialize("ACP actor runtime is missing".to_string()))?;
        self.resume_session_id(&runtime.conn, &cwd, session_id.clone()).await?;
        *runtime.expected_session_id.lock() = Some(session_id.clone());
        self.store_reusable_session(Some(session_id.clone()));
        Ok(SessionInfo { session_id })
    }

    async fn actor_set_session_mode(
        &self,
        runtime: &mut Option<ActorRuntime>,
        session_id: String,
        cwd: PathBuf,
        mode_id: String,
    ) -> Result<(), AcpError> {
        self.prepare_actor_runtime(runtime, cwd.clone()).await?;
        let runtime = runtime
            .as_mut()
            .ok_or_else(|| AcpError::Initialize("ACP actor runtime is missing".to_string()))?;
        self.resolve_existing_session(
            &runtime.conn,
            &cwd,
            session_id.clone(),
            &runtime.expected_session_id,
        )
        .await?;
        runtime
            .conn
            .set_session_mode(acp::SetSessionModeRequest::new(
                acp::SessionId::new(session_id.clone()),
                mode_id,
            ))
            .await
            .map_err(|err| AcpError::Prompt(err.to_string()))?;
        self.store_reusable_session(Some(session_id));
        Ok(())
    }

    async fn actor_set_session_config_option(
        &self,
        runtime: &mut Option<ActorRuntime>,
        session_id: String,
        cwd: PathBuf,
        option_name: String,
        value_id: String,
    ) -> Result<acp::SetSessionConfigOptionResponse, AcpError> {
        self.prepare_actor_runtime(runtime, cwd.clone()).await?;
        let runtime = runtime
            .as_mut()
            .ok_or_else(|| AcpError::Initialize("ACP actor runtime is missing".to_string()))?;
        self.resolve_existing_session(
            &runtime.conn,
            &cwd,
            session_id.clone(),
            &runtime.expected_session_id,
        )
        .await?;
        let error_context = format!(r#"for "{}"="{}""#, option_name, value_id);
        let response = runtime
            .conn
            .set_session_config_option(acp::SetSessionConfigOptionRequest::new(
                acp::SessionId::new(session_id.clone()),
                option_name,
                value_id,
            ))
            .await
            .map_err(|err| {
                wrap_session_control_error(
                    "session/set_config_option",
                    Some(error_context),
                    err,
                    AcpError::SetSessionConfigOption,
                )
            })?;
        self.store_reusable_session(Some(session_id));
        Ok(response)
    }

    async fn actor_set_session_model(
        &self,
        runtime: &mut Option<ActorRuntime>,
        session_id: String,
        cwd: PathBuf,
        model: String,
    ) -> Result<(), AcpError> {
        self.prepare_actor_runtime(runtime, cwd.clone()).await?;
        let runtime = runtime
            .as_mut()
            .ok_or_else(|| AcpError::Initialize("ACP actor runtime is missing".to_string()))?;
        self.resolve_existing_session(
            &runtime.conn,
            &cwd,
            session_id.clone(),
            &runtime.expected_session_id,
        )
        .await?;
        let error_context = format!(r#"for model "{}""#, model);
        runtime
            .conn
            .set_session_model(acp::SetSessionModelRequest::new(
                acp::SessionId::new(session_id.clone()),
                model,
            ))
            .await
            .map_err(|err| {
                wrap_session_control_error(
                    "session/set_model",
                    Some(error_context),
                    err,
                    AcpError::SetSessionModel,
                )
            })?;
        self.store_reusable_session(Some(session_id));
        Ok(())
    }

    async fn actor_run_prompt(
        &self,
        runtime: &mut Option<ActorRuntime>,
        request: PromptRequest,
        event_tx: mpsc::UnboundedSender<PromptEvent>,
    ) -> Result<PromptResult, AcpError> {
        *self.permission_stats.lock() = PermissionStats::default();
        self.prepare_actor_runtime(runtime, request.cwd.clone()).await?;
        let runtime = runtime
            .as_mut()
            .ok_or_else(|| AcpError::Initialize("ACP actor runtime is missing".to_string()))?;

        while runtime.event_rx.try_recv().is_ok() {}

        let session_id = self
            .resolve_session(
                &runtime.conn,
                &request.cwd,
                &request.session_strategy,
                &runtime.expected_session_id,
            )
            .await?;
        self.store_reusable_session(Some(session_id.clone()));

        let (cancel_tx, mut cancel_rx) = watch::channel(false);
        let (completed_tx, completed_rx) = watch::channel(false);
        self.register_active_prompt(session_id.clone(), cancel_tx, completed_rx);

        let prompt_future = runtime.conn.prompt(acp::PromptRequest::new(
            acp::SessionId::new(session_id.clone()),
            vec![request.prompt.into()],
        ));
        tokio::pin!(prompt_future);

        let mut deltas = Vec::new();
        let mut prompt_error = None::<AcpError>;
        let mut finish_reason = None;
        let mut usage = None;
        let mut prompt_finished = false;
        let mut cancel_sent = false;

        'prompt_loop: loop {
            if prompt_finished {
                while let Ok(event) = runtime.event_rx.try_recv() {
                    match event {
                        InternalEvent::Delta(delta) => {
                            if !delta.is_empty() {
                                deltas.push(delta.clone());
                                let _ = event_tx.send(PromptEvent::TextDelta(delta));
                            }
                        }
                        InternalEvent::SessionChanged { expected, actual } => {
                            let _ = event_tx.send(PromptEvent::SessionChanged {
                                expected: expected.clone(),
                                actual: actual.clone(),
                            });
                            prompt_error = Some(AcpError::SessionChanged { expected, actual });
                            break 'prompt_loop;
                        }
                    }
                }
                break;
            }

            tokio::select! {
                joined = &mut prompt_future, if !prompt_finished => {
                    prompt_finished = true;
                    match joined {
                        Ok(response) => {
                            finish_reason = Some(acp_finish_reason(response.stop_reason));
                            usage = response.usage.as_ref().map(map_usage);
                        }
                        Err(err) => {
                            prompt_error = Some(AcpError::Prompt(err.to_string()));
                        }
                    }
                }
                cancel_changed = cancel_rx.changed(), if !cancel_sent => {
                    match cancel_changed {
                        Ok(_) if *cancel_rx.borrow() => {
                            self.cancelling_session_ids.lock().insert(session_id.clone());
                            if let Err(err) = runtime.conn.cancel(acp::CancelNotification::new(acp::SessionId::new(session_id.clone()))).await {
                                prompt_error = Some(AcpError::Cancel(err.to_string()));
                                break 'prompt_loop;
                            }
                            cancel_sent = true;
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
                maybe_event = runtime.event_rx.recv() => {
                    match maybe_event {
                        Some(InternalEvent::Delta(delta)) => {
                            if !delta.is_empty() {
                                deltas.push(delta.clone());
                                let _ = event_tx.send(PromptEvent::TextDelta(delta));
                            }
                        }
                        Some(InternalEvent::SessionChanged { expected, actual }) => {
                            let _ = event_tx.send(PromptEvent::SessionChanged {
                                expected: expected.clone(),
                                actual: actual.clone(),
                            });
                            prompt_error = Some(AcpError::SessionChanged { expected, actual });
                            break 'prompt_loop;
                        }
                        None => {
                            if prompt_finished {
                                break;
                            }
                        }
                    }
                }
            }
        }

        let _ = completed_tx.send(true);
        self.cancelling_session_ids.lock().remove(&session_id);
        self.clear_active_prompt(&session_id);

        if let Some(err) = prompt_error {
            return Err(err);
        }

        Ok(PromptResult { session_id, deltas, finish_reason, usage })
    }

    fn actor_runtime_restart_reason(
        &self,
        runtime: &mut ActorRuntime,
        cwd: &Path,
    ) -> Option<&'static str> {
        if runtime.cwd != cwd {
            return Some("cwd_changed");
        }
        if runtime.io_task.is_finished() {
            return Some("io_closed");
        }
        match runtime.child.try_wait() {
            Ok(Some(_)) => Some("agent_exited"),
            Ok(None) => None,
            Err(err) => {
                tracing::warn!(
                    target: "vw_acp",
                    acp_agent = %self.agent_name,
                    error = %err,
                    "failed to query ACP agent process state before reusing runtime"
                );
                Some("child_wait_error")
            }
        }
    }

    async fn prepare_actor_runtime(
        &self,
        runtime: &mut Option<ActorRuntime>,
        cwd: PathBuf,
    ) -> Result<(), AcpError> {
        let restart_reason = runtime
            .as_mut()
            .and_then(|active_runtime| self.actor_runtime_restart_reason(active_runtime, &cwd));

        if let Some(reason) = restart_reason {
            if let Some(active_runtime) = runtime.take() {
                self.shutdown_actor_runtime(active_runtime, Some(reason), false).await;
            }
        } else if runtime.is_some() {
            return Ok(());
        }

        *runtime = Some(self.spawn_actor_runtime(&cwd).await?);
        Ok(())
    }

    async fn spawn_actor_runtime(&self, cwd: &Path) -> Result<ActorRuntime, AcpError> {
        let ProcessHandles { mut child, stderr_task } = self.spawn_child()?;
        let pid = child.id();
        let outgoing = TappedWriter::new(
            child.stdin.take().ok_or(AcpError::MissingStdin)?,
            self.make_message_tap(AcpMessageDirection::Outbound),
        )
        .compat_write();
        let expected_session_id = Arc::new(Mutex::new(None::<String>));
        let (event_tx, event_rx) = mpsc::unbounded_channel::<InternalEvent>();
        let incoming = TappedReader::new(
            child.stdout.take().ok_or(AcpError::MissingStdout)?,
            self.make_message_tap(AcpMessageDirection::Inbound),
        )
        .compat();
        let (conn, handle_io) = acp::ClientSideConnection::new(
            self.build_event_client(cwd, expected_session_id.clone(), event_tx),
            outgoing,
            incoming,
            |fut| {
                tokio::task::spawn_local(fut);
            },
        );
        let (io_closed_tx, io_closed_rx) = oneshot::channel();
        let io_task = tokio::task::spawn_local(async move {
            let _ = handle_io.await;
            let _ = io_closed_tx.send(());
        });

        match self.initialize_connection(&conn).await {
            Ok(()) => {
                self.record_actor_start(pid);
                Ok(ActorRuntime {
                    cwd: cwd.to_path_buf(),
                    child,
                    stderr_task,
                    expected_session_id,
                    event_rx,
                    conn,
                    io_closed_rx,
                    io_task,
                })
            }
            Err(err) => {
                self.shutdown_io_task(io_task).await;
                let finalized = self.finalize_child(child, stderr_task).await;
                Err(enrich_acp_error_with_process_context(err, &finalized))
            }
        }
    }

    async fn shutdown_actor_runtime(
        &self,
        runtime: ActorRuntime,
        reason: Option<&str>,
        unexpected_during_prompt: bool,
    ) -> FinalizedChild {
        let ActorRuntime {
            cwd: _,
            child,
            stderr_task,
            expected_session_id,
            event_rx,
            conn,
            io_closed_rx,
            io_task,
        } = runtime;
        drop(conn);
        drop(event_rx);
        drop(expected_session_id);
        drop(io_closed_rx);
        self.shutdown_io_task(io_task).await;
        let finalized = self.finalize_child(child, stderr_task).await;
        self.record_actor_exit(finalized.summary.clone(), reason, unexpected_during_prompt);
        self.store_reusable_session(None);
        finalized
    }

    fn record_actor_start(&self, pid: Option<u32>) {
        let mut state = self.actor_state.lock();
        state.lifecycle.pid = pid;
        state.lifecycle.started_at = Some(iso_now());
        state.lifecycle.last_exit = None;
    }

    fn record_actor_exit(
        &self,
        exit: ChildExitSummary,
        reason: Option<&str>,
        unexpected_during_prompt: bool,
    ) {
        let mut state = self.actor_state.lock();
        state.lifecycle.pid = None;
        state.lifecycle.last_exit = Some(AgentLifecycleExit {
            exit_code: exit.exit_code,
            signal: exit.signal,
            exited_at: Some(iso_now()),
            reason: reason.map(ToOwned::to_owned),
            unexpected_during_prompt,
        });
    }

    fn store_reusable_session(&self, session_id: Option<String>) {
        self.actor_state.lock().reusable_session_id = session_id;
    }

    /// 启动 ACP 代理子进程并返回进程句柄。
    ///
    /// 该函数会合并配置环境变量和认证环境变量，创建可管控的 stdin/stdout/stderr
    /// 管道。命令为空或进程启动失败时返回 [`AcpError`]。在 Unix 上，子进程会被
    /// 放入独立进程组，便于后续关闭时收敛整组进程。
    pub(crate) fn spawn_child(&self) -> Result<ProcessHandles, AcpError> {
        if self.config.command.trim().is_empty() {
            return Err(AcpError::EmptyCommand);
        }

        let mut env = self.config.env.clone();
        for (key, value) in build_agent_environment(&self.auth_credentials) {
            env.entry(key).or_insert(value);
        }

        let mut cmd = build_spawn_command(self.config.command.trim(), &env);
        cmd.args(self.config.args.iter())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }

        tracing::info!(
            target: "vw_acp",
            acp_agent = %self.agent_name,
            command = %self.config.command.trim(),
            args_count = self.config.args.len(),
            env_count = env.len(),
            "starting ACP agent process"
        );

        let mut child = cmd.spawn().map_err(AcpError::Spawn)?;
        let stderr_task = child.stderr.take().map(|mut stderr| {
            tokio::spawn(async move {
                let mut output = String::new();
                let _ = stderr.read_to_string(&mut output).await;
                output
            })
        });

        Ok(ProcessHandles { child, stderr_task })
    }

    async fn initialize_connection(
        &self,
        conn: &acp::ClientSideConnection,
    ) -> Result<(), AcpError> {
        let capabilities = acp::ClientCapabilities::new()
            .fs(acp::FileSystemCapabilities::new().read_text_file(true).write_text_file(true))
            .terminal(true);
        let response = conn.initialize(
            acp::InitializeRequest::new(acp::ProtocolVersion::V1)
                .client_info(
                    acp::Implementation::new(self.client_name.clone(), self.client_version.clone())
                        .title("VibeWindow ACP Client"),
                )
                .client_capabilities(capabilities),
        );
        let response = if self.is_gemini_acp_command() {
            timeout(resolve_gemini_acp_startup_timeout(), response)
                .await
                .map_err(|_| {
                    AcpError::GeminiStartupTimeout(build_gemini_acp_startup_timeout_message(
                        &self.config.command,
                    ))
                })?
                .map_err(|err| AcpError::Initialize(err.to_string()))?
        } else {
            response.await.map_err(|err| AcpError::Initialize(err.to_string()))?
        };
        self.authenticate_if_required(conn, &response.auth_methods).await?;
        Ok(())
    }

    async fn resolve_session(
        &self,
        conn: &acp::ClientSideConnection,
        cwd: &Path,
        session_strategy: &SessionStrategy,
        expected_session_id: &Arc<Mutex<Option<String>>>,
    ) -> Result<String, AcpError> {
        let session_id = match session_strategy {
            SessionStrategy::New => self.new_session_id(conn, cwd).await?,
            SessionStrategy::Load(session_id) => {
                self.load_session_id(conn, cwd, session_id.clone()).await?
            }
            SessionStrategy::Resume(session_id) => {
                self.resume_session_id(conn, cwd, session_id.clone()).await?
            }
            SessionStrategy::ResumeOrLoad(session_id) => {
                match self.resume_session_id(conn, cwd, session_id.clone()).await {
                    Ok(session_id) => session_id,
                    Err(_) => self.load_session_id(conn, cwd, session_id.clone()).await?,
                }
            }
            SessionStrategy::ResumeLoadOrNew(session_id) => {
                match self.resume_session_id(conn, cwd, session_id.clone()).await {
                    Ok(session_id) => session_id,
                    Err(_) => match self.load_session_id(conn, cwd, session_id.clone()).await {
                        Ok(session_id) => session_id,
                        Err(_) => self.new_session_id(conn, cwd).await?,
                    },
                }
            }
            SessionStrategy::LoadOrNew(session_id) => {
                match self.load_session_id(conn, cwd, session_id.clone()).await {
                    Ok(session_id) => session_id,
                    Err(_) => self.new_session_id(conn, cwd).await?,
                }
            }
        };

        *expected_session_id.lock() = Some(session_id.clone());
        Ok(session_id)
    }

    async fn resolve_existing_session(
        &self,
        conn: &acp::ClientSideConnection,
        cwd: &Path,
        session_id: String,
        expected_session_id: &Arc<Mutex<Option<String>>>,
    ) -> Result<(), AcpError> {
        match self.resume_session_id(conn, cwd, session_id.clone()).await {
            Ok(resolved) => {
                *expected_session_id.lock() = Some(resolved);
                Ok(())
            }
            Err(_) => {
                let resolved = self.load_session_id(conn, cwd, session_id).await?;
                *expected_session_id.lock() = Some(resolved);
                Ok(())
            }
        }
    }

    async fn new_session_id(
        &self,
        conn: &acp::ClientSideConnection,
        cwd: &Path,
    ) -> Result<String, AcpError> {
        let mut request = acp::NewSessionRequest::new(cwd).mcp_servers(self.mcp_servers.clone());
        if let Some(meta) = build_session_options_meta(self.session_options.as_ref()) {
            request = request.meta(meta);
        }
        let session = if self.is_claude_acp_command() {
            timeout(resolve_claude_acp_session_create_timeout(), conn.new_session(request))
                .await
                .map_err(|_| {
                    AcpError::ClaudeSessionCreateTimeout(
                        build_claude_acp_session_create_timeout_message(),
                    )
                })?
                .map_err(|err| AcpError::NewSession(err.to_string()))?
        } else {
            conn.new_session(request).await.map_err(|err| AcpError::NewSession(err.to_string()))?
        };
        Ok(session.session_id.0.to_string())
    }

    async fn load_session_id(
        &self,
        conn: &acp::ClientSideConnection,
        cwd: &Path,
        session_id: String,
    ) -> Result<String, AcpError> {
        conn.load_session(
            acp::LoadSessionRequest::new(session_id.clone(), cwd)
                .mcp_servers(self.mcp_servers.clone()),
        )
        .await
        .map_err(|err| AcpError::LoadSession(err.to_string()))?;
        Ok(session_id)
    }

    async fn resume_session_id(
        &self,
        conn: &acp::ClientSideConnection,
        cwd: &Path,
        session_id: String,
    ) -> Result<String, AcpError> {
        conn.resume_session(
            acp::ResumeSessionRequest::new(session_id.clone(), cwd)
                .mcp_servers(self.mcp_servers.clone()),
        )
        .await
        .map_err(|err| AcpError::ResumeSession(err.to_string()))?;
        Ok(session_id)
    }

    /// 收尾 ACP 代理子进程并收集退出摘要。
    ///
    /// 函数先短暂等待自然退出，再依次发送终止和强制结束信号，最后读取 stderr
    /// 片段用于错误诊断。返回值不会包含敏感环境变量，只保留退出码、信号和
    /// stderr 文本供上层决定是否拼接到错误信息。
    pub(crate) async fn finalize_child(
        &self,
        mut child: Child,
        stderr_task: Option<tokio::task::JoinHandle<String>>,
    ) -> FinalizedChild {
        let process_group_id = child.id();
        let graceful_timeout = Duration::from_millis(500);
        let status = match timeout(graceful_timeout, child.wait()).await {
            Ok(Ok(status)) => Some(status),
            _ => {
                send_terminate_signal_to_process_group(process_group_id);
                match timeout(Duration::from_secs(1), child.wait()).await {
                    Ok(Ok(status)) => Some(status),
                    Ok(Err(err)) => {
                        tracing::warn!(
                            target: "vw_acp",
                            acp_agent = %self.agent_name,
                            error = %err,
                            "failed to wait for ACP agent process exit"
                        );
                        None
                    }
                    Err(_) => {
                        send_kill_signal_to_process_group(process_group_id);
                        let _ = timeout(Duration::from_millis(500), child.wait()).await;
                        tracing::warn!(
                            target: "vw_acp",
                            acp_agent = %self.agent_name,
                            "timed out waiting for ACP agent process exit"
                        );
                        None
                    }
                }
            }
        };
        cleanup_process_group(process_group_id).await;

        let stderr_output = match stderr_task {
            Some(task) => match timeout(Duration::from_millis(300), task).await {
                Ok(Ok(output)) => output,
                Ok(Err(_)) | Err(_) => String::new(),
            },
            None => String::new(),
        };

        let summary = child_exit_summary(status.as_ref());

        if stderr_output.trim().is_empty() {
            tracing::debug!(
                target: "vw_acp",
                acp_agent = %self.agent_name,
                exit_code = status.and_then(|value| value.code()).unwrap_or_default(),
                "ACP agent process exited"
            );
            return FinalizedChild { summary, stderr_output };
        }

        let stderr_preview: String = stderr_output.chars().take(400).collect();
        if self.verbose {
            tracing::warn!(
                target: "vw_acp",
                acp_agent = %self.agent_name,
                exit_code = status.and_then(|value| value.code()).unwrap_or_default(),
                stderr_len = stderr_output.len(),
                stderr = %stderr_output,
                "ACP agent process wrote to stderr"
            );
            return FinalizedChild { summary, stderr_output };
        }

        tracing::warn!(
            target: "vw_acp",
            acp_agent = %self.agent_name,
            exit_code = status.and_then(|value| value.code()).unwrap_or_default(),
            stderr_len = stderr_output.len(),
            stderr = %stderr_preview,
            "ACP agent process wrote to stderr"
        );
        FinalizedChild { summary, stderr_output }
    }

    fn build_event_client(
        &self,
        cwd: &Path,
        expected_session_id: Arc<Mutex<Option<String>>>,
        event_tx: mpsc::UnboundedSender<InternalEvent>,
    ) -> AcpEventClient {
        let on_operation = self.on_client_operation.clone();
        AcpEventClient {
            expected_session_id,
            event_tx,
            filesystem: FileSystemHandlers::new(FileSystemHandlersOptions {
                cwd: cwd.to_path_buf(),
                permission_mode: self.permission_mode,
                non_interactive_permissions: self.non_interactive_permissions,
                on_operation: on_operation.clone(),
                confirm_write: None,
            }),
            terminal_manager: TerminalManager::new(TerminalManagerOptions {
                cwd: cwd.to_path_buf(),
                permission_mode: self.permission_mode,
                non_interactive_permissions: self.non_interactive_permissions,
                on_operation,
                confirm_execute: None,
                kill_grace_ms: None,
            }),
            permission_mode: self.permission_mode,
            non_interactive_permissions: self.non_interactive_permissions,
            on_session_update: self.on_session_update.clone(),
            permission_stats: self.permission_stats.clone(),
            cancelling_session_ids: self.cancelling_session_ids.clone(),
        }
    }

    fn make_message_tap(&self, direction: AcpMessageDirection) -> MessageTap {
        MessageTap::new(direction, self.on_acp_message.clone(), self.on_acp_output_message.clone())
    }

    async fn authenticate_if_required(
        &self,
        conn: &acp::ClientSideConnection,
        methods: &[acp::AuthMethod],
    ) -> Result<(), AcpError> {
        if methods.is_empty() {
            return Ok(());
        }

        let Some(selected) = self.select_auth_method(methods) else {
            if self.auth_policy == AuthPolicy::Fail {
                let method_ids = methods
                    .iter()
                    .map(|method| method.id().0.as_ref())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(AcpError::Initialize(format!(
                    "agent advertised auth methods [{method_ids}] but no matching credentials found"
                )));
            }

            if self.verbose {
                let method_ids = methods
                    .iter()
                    .map(|method| method.id().0.as_ref())
                    .collect::<Vec<_>>()
                    .join(", ");
                tracing::info!(
                    target: "vw_acp",
                    acp_agent = %self.agent_name,
                    auth_methods = %method_ids,
                    "agent advertised auth methods but no matching credentials were found; skipping client authentication"
                );
            }
            return Ok(());
        };

        conn.authenticate(acp::AuthenticateRequest::new(selected.method_id.clone()))
            .await
            .map_err(|err| AcpError::Initialize(err.to_string()))?;

        if self.verbose {
            tracing::info!(
                target: "vw_acp",
                acp_agent = %self.agent_name,
                method_id = %selected.method_id,
                source = selected.source,
                "authenticated ACP client"
            );
        }

        Ok(())
    }

    fn select_auth_method(&self, methods: &[acp::AuthMethod]) -> Option<AuthSelection> {
        for method in methods {
            let method_id = method.id().0.as_ref();

            if read_env_credential(method_id).is_some() {
                return Some(AuthSelection { method_id: method_id.to_string(), source: "env" });
            }

            let normalized = to_env_token(method_id);
            let config_credential = self
                .auth_credentials
                .get(method_id)
                .or_else(|| normalized.as_ref().and_then(|key| self.auth_credentials.get(key)));

            if config_credential.is_some_and(|value| !value.trim().is_empty()) {
                return Some(AuthSelection { method_id: method_id.to_string(), source: "config" });
            }
        }

        None
    }

    fn register_active_prompt(
        &self,
        session_id: String,
        cancel_tx: watch::Sender<bool>,
        completed_rx: watch::Receiver<bool>,
    ) {
        *self.active_prompt.lock() =
            Some(ActivePromptControl { session_id, cancel_tx, completed_rx });
    }

    fn clear_active_prompt(&self, session_id: &str) {
        let should_clear = self
            .active_prompt
            .lock()
            .as_ref()
            .is_some_and(|active_prompt| active_prompt.session_id == session_id);
        if should_clear {
            *self.active_prompt.lock() = None;
        }
    }

    fn is_gemini_acp_command(&self) -> bool {
        basename_token(&self.config.command) == "gemini"
            && self.config.args.iter().any(|arg| arg == "--acp" || arg == "--experimental-acp")
    }

    fn is_claude_acp_command(&self) -> bool {
        let command_token = basename_token(&self.config.command);
        command_token == "claude-agent-acp"
            || self.config.args.iter().any(|arg| arg.contains("claude-agent-acp"))
    }
}

#[cfg(test)]
#[path = "actor_tests.rs"]
mod actor_tests;
