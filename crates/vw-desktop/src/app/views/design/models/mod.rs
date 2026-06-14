use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorFormat {
    #[default]
    Hex,
    Rgba,
    Hsl,
    Css,
}

impl std::fmt::Display for ColorFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorFormat::Hex => write!(f, "HEX"),
            ColorFormat::Rgba => write!(f, "RGBA"),
            ColorFormat::Hsl => write!(f, "HSL"),
            ColorFormat::Css => write!(f, "CSS"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorPickerTarget {
    Fill { element_id: String, fill_index: usize },
    GradientStop { element_id: String, fill_index: usize, stop_index: usize },
    MeshPoint { element_id: String, fill_index: usize, point_index: usize },
    Effect { element_id: String, effect_index: usize },
    ContextFill { element_id: String },
    ContextBorder { element_id: String },
    ContextText { element_id: String },
    VariableValue { variable_name: String, mode: Option<String> },
    // Future extensions: Stroke, Shadow, Text, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDef {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub collection: Option<String>,
    #[serde(deserialize_with = "deserialize_variable_values")]
    pub value: Vec<VariableValue>,
}

fn deserialize_variable_values<'de, D>(deserializer: D) -> Result<Vec<VariableValue>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                let val: VariableValue =
                    serde_json::from_value(item).map_err(serde::de::Error::custom)?;
                result.push(val);
            }
            Ok(result)
        }
        serde_json::Value::String(s) => Ok(vec![VariableValue { value: s, theme: None }]),
        serde_json::Value::Number(n) => {
            Ok(vec![VariableValue { value: n.to_string(), theme: None }])
        }
        _ => Err(serde::de::Error::custom("Invalid variable value")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableValue {
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub value: String,
    #[serde(default, deserialize_with = "deserialize_theme_condition_option")]
    pub theme: Option<ThemeCondition>,
}

fn deserialize_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        _ => Err(serde::de::Error::custom("Expected string or number")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeCondition {
    #[serde(rename = "Mode")]
    pub mode: String,
}

fn deserialize_theme_condition_option<'de, D>(
    deserializer: D,
) -> Result<Option<ThemeCondition>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(s) => {
            let mode = s.trim();
            if mode.is_empty() {
                Ok(None)
            } else {
                Ok(Some(ThemeCondition { mode: mode.to_string() }))
            }
        }
        serde_json::Value::Object(map) => {
            let mode = map
                .get("Mode")
                .or_else(|| map.get("mode"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            if mode.is_empty() { Ok(None) } else { Ok(Some(ThemeCondition { mode })) }
        }
        _ => Ok(None),
    }
}

fn deserialize_children_lenient<'de, D>(deserializer: D) -> Result<Vec<DesignElement>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    let mut children = Vec::new();
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                if let Ok(child) = serde_json::from_value::<DesignElement>(item) {
                    children.push(child);
                }
            }
        }
        serde_json::Value::Object(object) => {
            if let Ok(child) =
                serde_json::from_value::<DesignElement>(serde_json::Value::Object(object))
            {
                children.push(child);
            }
        }
        _ => {}
    }
    Ok(children)
}

fn deserialize_stroke_option<'de, D>(deserializer: D) -> Result<Option<Stroke>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: serde_json::Value = serde::Deserialize::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(fill) => {
            let fill = fill.trim();
            if fill.is_empty() {
                Ok(None)
            } else {
                Ok(Some(Stroke { align: None, thickness: None, fill: Some(fill.to_string()) }))
            }
        }
        serde_json::Value::Object(object) => {
            let align =
                object.get("align").and_then(|value| value.as_str()).map(ToString::to_string);
            let thickness =
                object.get("thickness").cloned().or_else(|| object.get("width").cloned());
            let fill = object
                .get("fill")
                .or_else(|| object.get("color"))
                .and_then(|value| value.as_str())
                .map(ToString::to_string);
            Ok(Some(Stroke { align, thickness, fill }))
        }
        _ => Ok(None),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DesignTool {
    #[default]
    Move,
    Line,
    Rectangle,
    Ellipse,
    Triangle,
    Diamond,
    Star,
    Pentagon,
    Hexagon,
    Parallelogram,
    Trapezoid,
    Chevron,
    Capsule,
    Icon,
    ImportImage,
    ImportFigma,
    Pen,
    Eraser,
    Text,
    Frame,
    StickyNote,
    Hand,
}

impl DesignTool {
    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Move => "cursor-fill.svg",
            Self::Line => "minus.svg",
            Self::Rectangle => "square.svg",
            Self::Ellipse => "circle.svg",
            Self::Triangle => "triangle.svg",
            Self::Diamond => "diamond.svg",
            Self::Star => "star.svg",
            Self::Pentagon => "pentagon.svg",
            Self::Hexagon => "hexagon.svg",
            Self::Parallelogram => "parallelogram.svg",
            Self::Trapezoid => "trapezoid.svg",
            Self::Chevron => "chevron.svg",
            Self::Capsule => "capsule.svg",
            Self::Icon => "gem.svg",
            Self::ImportImage => "image.svg",
            Self::ImportFigma => "file-code.svg",
            Self::Pen => "pen.svg",
            Self::Eraser => "eraser.svg",
            Self::Text => "fonts.svg",
            Self::Frame => "bounding-box.svg",
            Self::StickyNote => "sticky.svg",
            Self::Hand => "arrows-move.svg",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum StickyNoteKind {
    #[default]
    Note,
    Context,
    Prompt,
}

impl StickyNoteKind {
    pub const ALL: [Self; 3] = [Self::Note, Self::Context, Self::Prompt];

    pub fn label(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Context => "Context",
            Self::Prompt => "Prompt",
        }
    }

    pub fn label_zh(self) -> &'static str {
        match self {
            Self::Note => "笔记",
            Self::Context => "上下文",
            Self::Prompt => "提示词",
        }
    }

    pub fn bilingual_label(self) -> String {
        format!("{} {}", self.label(), self.label_zh())
    }

    pub fn fill_color(self) -> &'static str {
        match self {
            Self::Note => "#FFF1CC",
            Self::Context => "#FFFFFF",
            Self::Prompt => "#DFF0FF",
        }
    }

    pub fn stroke_color(self) -> &'static str {
        match self {
            Self::Note => "#A67A2D",
            Self::Context => "#8A8A8A",
            Self::Prompt => "#0D8BFF",
        }
    }

    pub fn text_color(self) -> &'static str {
        match self {
            Self::Note => "#6E4A12",
            Self::Context => "#3A3F46",
            Self::Prompt => "#0B6FBD",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "note" => Some(Self::Note),
            "context" => Some(Self::Context),
            "prompt" => Some(Self::Prompt),
            _ => None,
        }
    }

    pub fn from_value(value: &serde_json::Value) -> Option<Self> {
        value.as_str().and_then(Self::from_str)
    }
}

impl std::fmt::Display for StickyNoteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.bilingual_label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesignThemes {
    #[serde(rename = "Mode", default)]
    pub mode: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VariableCollections {
    #[serde(default)]
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesignGroup {
    #[serde(default)]
    pub id: u32,
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesignDoc {
    pub version: String,
    #[serde(default)]
    pub children: Vec<DesignElement>,
    #[serde(default)]
    pub variables: HashMap<String, VariableDef>,
    #[serde(default)]
    pub groups: Vec<DesignGroup>,
    #[serde(default, deserialize_with = "deserialize_theme_condition_option")]
    pub theme: Option<ThemeCondition>,
    #[serde(default)]
    pub themes: Option<DesignThemes>,
    #[serde(default)]
    pub variable_collections: Option<VariableCollections>,
    #[serde(skip)]
    pub images: HashMap<String, iced::widget::image::Handle>,
    #[serde(skip)]
    pub image_sizes: HashMap<String, (u32, u32)>,
    #[serde(skip)]
    pub tailwind_selection: Option<(String, Vec<usize>)>,
}

impl DesignDoc {
    pub fn default_group_name(id: u32) -> String {
        if id == 0 { "默认页面".to_string() } else { format!("页面 {}", id) }
    }

    pub fn variable_collection_names(&self) -> Vec<String> {
        fn push_unique(names: &mut Vec<String>, name: &str) {
            let trimmed = name.trim();
            if trimmed.is_empty() {
                return;
            }
            if names.iter().any(|item| item.eq_ignore_ascii_case(trimmed)) {
                return;
            }
            names.push(trimmed.to_string());
        }

        let mut names = Vec::new();

        if let Some(collections) = &self.variable_collections {
            for name in &collections.names {
                push_unique(&mut names, name);
            }
        }

        for def in self.variables.values() {
            if let Some(collection) = def.collection.as_deref() {
                push_unique(&mut names, collection);
            }
        }

        if names.is_empty() {
            names.push("Theme".to_string());
        }

        names
    }

    pub fn ensure_variable_collections(&mut self) -> Vec<String> {
        let names = self.variable_collection_names();
        self.variable_collections = Some(VariableCollections { names: names.clone() });
        names
    }

    pub fn variable_theme_modes(&self) -> Vec<String> {
        fn push_unique(modes: &mut Vec<String>, mode: &str) {
            let trimmed = mode.trim();
            if trimmed.is_empty() {
                return;
            }
            if modes.iter().any(|item| item.eq_ignore_ascii_case(trimmed)) {
                return;
            }
            modes.push(trimmed.to_string());
        }

        let mut modes = Vec::new();

        if let Some(themes) = &self.themes {
            for mode in &themes.mode {
                push_unique(&mut modes, mode);
            }
        }

        if let Some(theme) = &self.theme {
            push_unique(&mut modes, &theme.mode);
        }

        for def in self.variables.values() {
            for value in &def.value {
                if let Some(theme) = &value.theme {
                    push_unique(&mut modes, &theme.mode);
                }
            }
        }

        if modes.is_empty() {
            modes.push(
                self.theme
                    .as_ref()
                    .map(|theme| theme.mode.clone())
                    .unwrap_or_else(|| "Light".to_string()),
            );
        }

        modes
    }

    pub fn ensure_variable_themes(&mut self) -> Vec<String> {
        let modes = self.variable_theme_modes();
        self.themes = Some(DesignThemes { mode: modes.clone() });

        if let Some(current) = self.theme.as_mut()
            && current.mode.trim().is_empty()
        {
            current.mode = modes.first().cloned().unwrap_or_else(|| "Light".to_string());
        }

        modes
    }

    pub fn normalize_groups(&mut self) {
        fn collect_group_ids(elements: &[DesignElement], ids: &mut BTreeSet<u32>) {
            for element in elements {
                ids.insert(element.group_id);
                collect_group_ids(&element.children, ids);
            }
        }

        let mut used_group_ids = BTreeSet::new();
        collect_group_ids(&self.children, &mut used_group_ids);
        if used_group_ids.is_empty() {
            used_group_ids.insert(0);
        }

        let mut normalized_groups = Vec::with_capacity(self.groups.len().max(used_group_ids.len()));
        let mut seen_group_ids = BTreeSet::new();
        for mut group in self.groups.drain(..) {
            if !seen_group_ids.insert(group.id) {
                continue;
            }
            if group.name.trim().is_empty() {
                group.name = Self::default_group_name(group.id);
            }
            normalized_groups.push(group);
        }

        for group_id in used_group_ids {
            if !seen_group_ids.contains(&group_id) {
                normalized_groups
                    .push(DesignGroup { id: group_id, name: Self::default_group_name(group_id) });
            }
        }

        if normalized_groups.is_empty() {
            normalized_groups.push(DesignGroup { id: 0, name: Self::default_group_name(0) });
        }

        self.groups = normalized_groups;
    }

    pub fn first_group_id(&self) -> u32 {
        self.groups.first().map(|group| group.id).unwrap_or(0)
    }

    pub fn next_group_id(&self) -> u32 {
        self.groups.iter().map(|group| group.id).max().unwrap_or(0).saturating_add(1)
    }

    pub fn first_top_level_in_group(&self, group_id: u32) -> Option<&DesignElement> {
        self.children.iter().find(|child| child.group_id == group_id)
    }

    pub fn top_level_children_count_in_group(&self, group_id: u32) -> usize {
        self.children.iter().filter(|child| child.group_id == group_id).count()
    }

    pub fn group_name(&self, group_id: u32) -> Option<&str> {
        self.groups.iter().find(|group| group.id == group_id).map(|group| group.name.as_str())
    }

    pub fn group_id_for_element(&self, target_id: &str) -> Option<u32> {
        self.children.iter().find_map(|child| {
            if child.find_element(target_id).is_some() { Some(child.group_id) } else { None }
        })
    }

    pub fn filtered_for_group(&self, group_id: u32) -> Self {
        let mut filtered = self.clone();
        filtered.children =
            self.children.iter().filter(|child| child.group_id == group_id).cloned().collect();
        filtered
    }

    pub fn normalize_fill_flags(&mut self) {
        for child in &mut self.children {
            child.normalize_fill_flags();
        }
    }

    pub fn get_bounds(&self) -> Option<(f32, f32, f32, f32)> {
        if self.children.is_empty() {
            return None;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        let theme_mode = self.theme.as_ref().map(|t| t.mode.as_str());

        for child in &self.children {
            let x = child.x;
            let y = child.y;
            // Parse width/height safely
            let w = parse_val(&child.width, &self.variables, theme_mode).unwrap_or(0.0);
            let h = parse_val(&child.height, &self.variables, theme_mode).unwrap_or(0.0);

            if x < min_x {
                min_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if x + w > max_x {
                max_x = x + w;
            }
            if y + h > max_y {
                max_y = y + h;
            }
        }

        if min_x > max_x {
            return None;
        }

        Some((min_x, min_y, max_x, max_y))
    }

    pub fn find_element<'a>(&'a self, id: &str) -> Option<&'a DesignElement> {
        for child in &self.children {
            if let Some(el) = child.find_element(id) {
                return Some(el);
            }
        }
        None
    }

    pub fn find_path_to_element(&self, id: &str) -> Option<Vec<String>> {
        for child in &self.children {
            let mut path = Vec::new();
            if child.find_path(id, &mut path) {
                return Some(path);
            }
        }
        None
    }

    pub fn update_property(&mut self, id: &str, key: &str, value: serde_json::Value) {
        for child in &mut self.children {
            if child.update_property(id, key, value.clone()) {
                return;
            }
        }
    }
}

pub fn compute_tree_metrics(doc: &DesignDoc) -> (usize, u16) {
    fn name_len(el: &DesignElement) -> usize {
        el.name.as_deref().filter(|s| !s.is_empty()).unwrap_or(el.kind.as_str()).chars().count()
    }

    fn children<'a>(doc: &'a DesignDoc, el: &'a DesignElement) -> &'a [DesignElement] {
        if el.kind == "ref"
            && let Some(ref_id) = &el.reference
            && let Some(ref_el) = doc.find_element(ref_id)
        {
            return &ref_el.children;
        }
        &el.children
    }

    fn walk(doc: &DesignDoc, el: &DesignElement, depth: u16, acc: &mut (usize, u16)) {
        let len = name_len(el);
        if len > acc.0 {
            acc.0 = len;
        }
        if depth > acc.1 {
            acc.1 = depth;
        }
        for child in children(doc, el) {
            walk(doc, child, depth + 1, acc);
        }
    }

    let mut acc = (0usize, 0u16);
    for c in &doc.children {
        walk(doc, c, 0, &mut acc);
    }
    acc
}

impl DesignElement {
    pub fn sticky_note_kind(&self) -> StickyNoteKind {
        self.note_type.unwrap_or_default()
    }

    pub fn set_group_id_recursive(&mut self, group_id: u32) {
        self.group_id = group_id;
        for child in &mut self.children {
            child.set_group_id_recursive(group_id);
        }
    }

    pub fn normalize_fill_flags(&mut self) {
        if is_fill_container_value(&self.width) {
            self.fill_width = Some(true);
        }
        if is_fill_container_value(&self.height) {
            self.fill_height = Some(true);
        }
        for child in &mut self.children {
            child.normalize_fill_flags();
        }
    }

    pub fn update_property(
        &mut self,
        target_id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> bool {
        if self.id == target_id {
            match key {
                "name" => self.name = value.as_str().map(|s| s.to_string()),
                "x" => self.x = value.as_f64().map(|v| v as f32).unwrap_or(self.x),
                "y" => self.y = value.as_f64().map(|v| v as f32).unwrap_or(self.y),
                "width" => self.width = Some(value),
                "height" => self.height = Some(value),
                "color" => self.color = value.as_str().map(|s| s.to_string()),
                "class" => self.class = value.as_str().map(|s| s.to_string()),
                "rotation" => self.rotation = value.as_f64().map(|v| v as f32),
                "content" => self.content = value.as_str().map(|s| s.to_string()),
                "context" => self.context = value.as_str().map(|s| s.to_string()),
                "noteType" => self.note_type = StickyNoteKind::from_value(&value),
                "fontFamily" => self.font_family = value.as_str().map(|s| s.to_string()),
                "fontSize" => self.font_size = Some(value),
                "fontWeight" => self.font_weight = Some(value),
                "weight" => self.weight = Some(value),
                "fontStyle" => self.font_style = value.as_str().map(|s| s.to_string()),
                "textDecoration" => self.text_decoration = value.as_str().map(|s| s.to_string()),
                "lineHeight" => self.line_height = Some(value),
                "letterSpacing" => self.letter_spacing = Some(value),
                "textAlign" => self.text_align = value.as_str().map(|s| s.to_string()),
                "textAlignVertical" => {
                    self.text_align_vertical = value.as_str().map(|s| s.to_string())
                }
                "textGrowth" => {
                    self.text_growth =
                        if value.is_null() { None } else { value.as_str().map(|s| s.to_string()) }
                }
                "fill" => self.fill = Some(value),
                "iconFontName" => self.icon_font_name = value.as_str().map(|s| s.to_string()),
                "iconFontFamily" => self.icon_font_family = value.as_str().map(|s| s.to_string()),
                "opacity" => self.opacity = value.as_f64().map(|v| v as f32),
                "fillWidth" => self.fill_width = value.as_bool(),
                "fillHeight" => self.fill_height = value.as_bool(),
                "visible" => self.visible = value.as_bool(),
                "effect" => self.effect = Some(value),
                "theme" => self.theme = Some(value),
                "export" => self.export = Some(value),
                "stroke" => {
                    if let Ok(s) = serde_json::from_value(value) {
                        self.stroke = Some(s);
                    }
                }
                _ => {}
            }
            return true;
        }
        for child in &mut self.children {
            if child.update_property(target_id, key, value.clone()) {
                return true;
            }
        }
        false
    }

    pub fn find_element<'a>(&'a self, target_id: &str) -> Option<&'a DesignElement> {
        if self.id == target_id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(el) = child.find_element(target_id) {
                return Some(el);
            }
        }
        None
    }

    pub fn find_path(&self, target_id: &str, path: &mut Vec<String>) -> bool {
        if self.id == target_id {
            return true;
        }

        path.push(self.id.clone());
        for child in &self.children {
            if child.find_path(target_id, path) {
                return true;
            }
        }
        path.pop();
        false
    }
}

pub fn resolve_variable<'a>(
    name: &str,
    variables: &'a HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Option<&'a String> {
    if let Some(def) = variables.get(name) {
        // Try to find a value for the current theme mode
        let val = if let Some(mode) = theme_mode {
            def.value.iter().find(|v| v.theme.as_ref().map(|t| t.mode == mode).unwrap_or(false))
        } else {
            None
        };

        // Fallback to default (no theme or first one)
        let val = val
            .or_else(|| def.value.iter().find(|v| v.theme.is_none()))
            .or_else(|| def.value.first());

        if let Some(v) = val {
            return Some(&v.value);
        }
    }
    None
}

pub fn parse_val(
    v: &Option<serde_json::Value>,
    variables: &HashMap<String, VariableDef>,
    theme_mode: Option<&str>,
) -> Option<f32> {
    match v {
        Some(serde_json::Value::Number(n)) => n.as_f64().map(|f| f as f32),
        Some(serde_json::Value::String(s)) => {
            if s.starts_with("$-") {
                let var_name = s.strip_prefix("$-").unwrap_or(s);
                if let Some(val_str) = resolve_variable(var_name, variables, theme_mode) {
                    return parse_val(
                        &Some(serde_json::Value::String(val_str.clone())),
                        variables,
                        theme_mode,
                    );
                }
            }

            if let Ok(val) = s.parse::<f32>() {
                Some(val)
            } else if s.starts_with("fill_container") {
                // Try to extract number
                let n_str = s.trim_start_matches("fill_container(").trim_end_matches(")");
                n_str.parse::<f32>().ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_fill_container_value(v: &Option<serde_json::Value>) -> bool {
    matches!(
        v,
        Some(serde_json::Value::String(s))
            if s == "fill_container" || s.starts_with("fill_container(")
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesignElement {
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub id: String,

    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default, rename = "groupId", alias = "group_id")]
    pub group_id: u32,
    pub name: Option<String>,
    pub width: Option<serde_json::Value>,
    pub height: Option<serde_json::Value>,
    pub fill: Option<serde_json::Value>,
    pub geometry: Option<String>,

    #[serde(default, deserialize_with = "deserialize_children_lenient")]
    pub children: Vec<DesignElement>,

    #[serde(rename = "ref")]
    pub reference: Option<String>,

    pub descendants: Option<serde_json::Value>,

    pub layout: Option<String>,
    pub gap: Option<serde_json::Value>,
    pub padding: Option<serde_json::Value>,
    pub slot: Option<serde_json::Value>,
    #[serde(rename = "alignItems")]
    pub align_items: Option<String>,
    #[serde(rename = "justifyContent")]
    pub justify_content: Option<String>,

    #[serde(rename = "cornerRadius")]
    pub corner_radius: Option<serde_json::Value>,

    #[serde(default, deserialize_with = "deserialize_stroke_option")]
    pub stroke: Option<Stroke>,
    pub effect: Option<serde_json::Value>,

    // Text specific (speculative)
    pub content: Option<String>,
    pub context: Option<String>,
    #[serde(rename = "noteType", alias = "note_type")]
    pub note_type: Option<StickyNoteKind>,
    #[serde(rename = "fontSize")]
    pub font_size: Option<serde_json::Value>,
    #[serde(rename = "fontFamily")]
    pub font_family: Option<String>,
    #[serde(rename = "fontWeight")]
    pub font_weight: Option<serde_json::Value>,
    #[serde(rename = "fontStyle")]
    pub font_style: Option<String>,
    #[serde(rename = "textDecoration")]
    pub text_decoration: Option<String>,
    #[serde(rename = "lineHeight")]
    pub line_height: Option<serde_json::Value>,
    #[serde(rename = "letterSpacing")]
    pub letter_spacing: Option<serde_json::Value>,
    #[serde(rename = "textAlignVertical")]
    pub text_align_vertical: Option<String>,
    #[serde(rename = "textAlign")]
    pub text_align: Option<String>,
    #[serde(rename = "textGrowth")]
    pub text_growth: Option<String>,
    pub color: Option<String>,
    #[serde(rename = "iconFontName", alias = "icon_font_name")]
    pub icon_font_name: Option<String>,
    #[serde(rename = "iconFontFamily", alias = "icon_font_family")]
    pub icon_font_family: Option<String>,
    #[serde(rename = "weight")]
    pub weight: Option<serde_json::Value>,

    // Tailwind support
    pub class: Option<String>,

    pub rotation: Option<f32>,
    pub opacity: Option<f32>,
    pub enabled: Option<bool>,

    // Other fields
    pub clip: Option<bool>,
    #[serde(rename = "clipContent", alias = "clip_content")]
    pub clip_content: Option<bool>,

    #[serde(rename = "fillWidth", alias = "fill_width")]
    pub fill_width: Option<bool>,
    #[serde(rename = "hugWidth", alias = "hug_width")]
    pub hug_width: Option<bool>,
    #[serde(rename = "fillHeight", alias = "fill_height")]
    pub fill_height: Option<bool>,
    #[serde(rename = "hugHeight", alias = "hug_height")]
    pub hug_height: Option<bool>,

    pub reusable: Option<bool>,
    pub visible: Option<bool>,
    pub theme: Option<serde_json::Value>,
    pub export: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub align: Option<String>,
    pub thickness: Option<serde_json::Value>,
    pub fill: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "shadowType")]
    pub shadow_type: Option<String>,
    pub color: Option<String>,
    pub offset: Option<Offset>,
    pub blur: Option<f32>,
    pub radius: Option<f32>,
    pub spread: Option<f32>,
    pub visible: Option<bool>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
