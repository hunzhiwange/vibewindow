# 测试反模式

**在以下情况下加载此参考：** 编写或修改测试、添加 mock，或忍不住想向生产代码添加仅测试用的方法时。

## 概述

测试必须验证真实行为，而非 mock 行为。Mock 是隔离的手段，不是被测试的对象。

**核心原则：** 测试代码做了什么，而不是 mock 做了什么。

**严格遵循 TDD 可以防止这些反模式。**

## 铁律

```
1. 绝不测试 mock 行为
2. 绝不向生产类添加仅测试用的方法
3. 绝不在不了解依赖的情况下使用 mock
```

## 反模式 1：测试 Mock 行为

**违规示例：**
```typescript
// ❌ 错误：测试 mock 是否存在
test('渲染侧边栏', () => {
  render(<Page />);
  expect(screen.getByTestId('sidebar-mock')).toBeInTheDocument();
});
```

**为什么这是错的：**
- 你在验证 mock 能工作，而不是组件能工作
- mock 存在时测试通过，不存在时失败
- 告诉你关于真实行为的任何信息为零

**你的人类搭档的纠正：** "我们是在测试 mock 的行为吗？"

**修正方法：**
```typescript
// ✅ 正确：测试真实组件，或者不要 mock 它
test('渲染侧边栏', () => {
  render(<Page />);  // 不要 mock sidebar
  expect(screen.getByRole('navigation')).toBeInTheDocument();
});

// 或者如果 sidebar 必须被 mock 以实现隔离：
// 不要对 mock 做断言 - 测试 Page 在 sidebar 存在时的行为
```

### 关卡函数

```
在对任何 mock 元素做断言之前：
  问："我是在测试真实组件行为还是仅仅测试 mock 是否存在？"

  如果是测试 mock 是否存在：
    停下 - 删除断言或取消 mock 该组件

  改为测试真实行为
```

## 反模式 2：生产代码中的仅测试方法

**违规示例：**
```typescript
// ❌ 错误：destroy() 只在测试中使用
class Session {
  async destroy() {  // 看起来像生产 API！
    await this._workspaceManager?.destroyWorkspace(this.id);
    // ... 清理
  }
}

// 在测试中
afterEach(() => session.destroy());
```

**为什么这是错的：**
- 生产类被仅测试用的代码污染
- 如果在生产中意外调用很危险
- 违反 YAGNI 原则和关注点分离
- 混淆了对象生命周期与实体生命周期

**修正方法：**
```typescript
// ✅ 正确：测试工具负责测试清理
// Session 没有 destroy() - 在生产中它是无状态的

// 在 test-utils/ 中
export async function cleanupSession(session: Session) {
  const workspace = session.getWorkspaceInfo();
  if (workspace) {
    await workspaceManager.destroyWorkspace(workspace.id);
  }
}

// 在测试中
afterEach(() => cleanupSession(session));
```

### 关卡函数

```
在向生产类添加任何方法之前：
  问："这个方法是否只在测试中使用？"

  如果是：
    停下 - 不要添加它
    把它放到测试工具中

  问："这个类是否拥有此资源的生命周期？"

  如果否：
    停下 - 这个方法放错了类
```

## 反模式 3：不了解情况就 Mock

**违规示例：**
```typescript
// ❌ 错误：Mock 破坏了测试逻辑
test('检测重复服务器', () => {
  // Mock 阻止了测试依赖的配置写入！
  vi.mock('ToolCatalog', () => ({
    discoverAndCacheTools: vi.fn().mockResolvedValue(undefined)
  }));

  await addServer(config);
  await addServer(config);  // 应该抛出异常 - 但不会！
});
```

**为什么这是错的：**
- 被 mock 的方法有测试依赖的副作用（写入配置）
- 过度 mock 以"保险"反而破坏了真实行为
- 测试因为错误原因通过或神秘地失败

**修正方法：**
```typescript
// ✅ 正确：在正确的层级 Mock
test('检测重复服务器', () => {
  // 只 mock 慢的部分，保留测试需要的行为
  vi.mock('MCPServerManager'); // 只 mock 慢的服务器启动

  await addServer(config);  // 配置被写入
  await addServer(config);  // 检测到重复 ✓
});
```

### 关卡函数

```
在 mock 任何方法之前：
  停下 - 还不要 mock

  1. 问："真实方法有什么副作用？"
  2. 问："这个测试是否依赖其中任何副作用？"
  3. 问："我是否完全理解这个测试需要什么？"

  如果依赖副作用：
    在更低层级 mock（实际的慢/外部操作）
    或使用保留必要行为的测试替身
    而不是测试依赖的高层方法

  如果不确定测试依赖什么：
    先用真实实现运行测试
    观察实际需要发生什么
    然后在正确的层级添加最少的 mock

  红旗：
    - "为了保险我 mock 一下"
    - "这可能很慢，还是 mock 吧"
    - 在不了解依赖链的情况下 mock
```

## 反模式 4：不完整的 Mock

**违规示例：**
```typescript
// ❌ 错误：部分 mock - 只有你认为需要的字段
const mockResponse = {
  status: 'success',
  data: { userId: '123', name: 'Alice' }
  // 缺失：下游代码使用的 metadata
};

// 后来：当代码访问 response.metadata.requestId 时出错
```

**为什么这是错的：**
- **部分 mock 隐藏了结构假设** - 你只 mock 了你已知的字段
- **下游代码可能依赖你没有包含的字段** - 静默失败
- **测试通过但集成失败** - Mock 不完整，真实 API 完整
- **虚假的信心** - 测试对真实行为证明不了什么

**铁律：** Mock 完整的数据结构，与现实存在的完全一致，而不是只 mock 你当前测试使用的字段。

**修正方法：**
```typescript
// ✅ 正确：映射真实 API 的完整性
const mockResponse = {
  status: 'success',
  data: { userId: '123', name: 'Alice' },
  metadata: { requestId: 'req-789', timestamp: 1234567890 }
  // 真实 API 返回的所有字段
};
```

### 关卡函数

```
在创建 mock 响应之前：
  检查："真实 API 响应包含哪些字段？"

  操作：
    1. 查看文档/示例中的实际 API 响应
    2. 包含系统可能在下游使用的所有字段
    3. 验证 mock 完全匹配真实响应模式

  关键：
    如果你在创建 mock，你必须理解整个结构
    部分 mock 在代码依赖被省略的字段时会静默失败

  如果不确定：包含所有有文档记录的字段
```

## 反模式 5：集成测试作为事后补充

**违规示例：**
```
✅ 实现完成
❌ 没有编写测试
"准备好测试了"
```

**为什么这是错的：**
- 测试是实现的一部分，不是可选的后续步骤
- TDD 本来可以捕获这个问题
- 没有测试就不能声称完成

**修正方法：**
```
TDD 循环：
1. 写失败测试
2. 实现使其通过
3. 重构
4. 然后才能声称完成
```

## 当 Mock 变得太复杂时

**警告信号：**
- Mock 设置比测试逻辑还长
- Mock 所有东西才能让测试通过
- Mock 缺少真实组件拥有的方法
- Mock 变更时测试就坏了

**你的人类搭档的问题：** "我们这里真的需要使用 mock 吗？"

**考虑：** 使用真实组件的集成测试通常比复杂的 mock 更简单

## TDD 防止这些反模式

**为什么 TDD 有帮助：**
1. **先写测试** → 迫使你思考你实际在测试什么
2. **看它失败** → 确认测试测试的是真实行为，不是 mock
3. **最少的实现** → 不会混入仅测试用的方法
4. **真实依赖** → 在 mock 之前你看到了测试实际需要什么

**如果你在测试 mock 行为，你就违反了 TDD** - 你在没有先看测试对真实代码失败的情况下就添加了 mock。

## 快速参考

| 反模式 | 修正方法 |
|--------|----------|
| 对 mock 元素做断言 | 测试真实组件或取消 mock |
| 生产代码中的仅测试方法 | 移到测试工具中 |
| 不了解情况就 mock | 先了解依赖，最少的 mock |
| 不完整的 mock | 完整映射真实 API |
| 测试作为事后补充 | TDD - 先写测试 |
| 过于复杂的 mock | 考虑集成测试 |

## 红旗

- 断言检查 `*-mock` 测试 ID
- 只在测试文件中调用的方法
- Mock 设置占测试的 >50%
- 移除 mock 时测试就失败
- 无法解释为什么需要 mock
- "为了保险而 mock"

## 总结

**Mock 是隔离的工具，不是测试的对象。**

如果 TDD 揭示了你正在测试 mock 行为，你就走偏了。

修正：测试真实行为，或质疑你为什么要用 mock。
