# Lunaris 设计约束

## 输出硬规则

- 只输出 JSON，不输出解释、Markdown 代码块、注释、`null`、`undefined`、尾逗号。
- 根对象只保留：`version`、`theme`、`variables`、`children`。
- `version` 固定为 `"2.6"`。
- `theme` 固定为 `{ "Mode": "Dark" }`。
- `children` 必须是数组。
- 优先复用 token，不要在深色界面里堆大量硬编码颜色。
- 高亮和发光只用于核心 CTA、焦点态、关键状态，不把整块背景做成强光面。
- 深色层级最多控制在三层：`$--background` → `$--card` → `$--secondary`。

## 允许元素与白名单

- 允许元素：`frame`、`text`、`rectangle`、`icon_font`、`ref`

优先级：

1. 页面和模块骨架优先用 `frame + text`
2. 面板底色、分隔线、进度条再补 `rectangle`
3. 工具动作、状态提示、导航切换再用 `icon_font`
4. 两处以上复用的结构才抽成 `reusable: true` 并通过 `ref` 继承

### `frame`

- `type` `id` `name` `x` `y` `width` `height` `fill`
- `children` `layout` `gap` `padding`
- `stroke` `cornerRadius` `clip` `reusable`

### `text`

- `type` `id` `name` `x` `y` `width` `height` `fill` `content`
- `fontFamily` `fontSize` `fontWeight` `lineHeight`
- `textAlign` `textGrowth`

### `rectangle`

- `type` `id` `name` `x` `y` `width` `height` `fill`
- `stroke` `cornerRadius`

### `icon_font`

- `type` `id` `name` `x` `y` `width` `height` `fill`
- `iconFontName` `iconFontFamily`

### `ref`

- `type` `id` `name` `x` `y` `width` `height`
- `ref` `descendants`

### `stroke`

- `align`
- `thickness`
- `fill`

## 每种元素的最小 JSON 结构

### 根对象最小结构

```json
{
  "version": "2.6",
  "theme": { "Mode": "Dark" },
  "variables": {},
  "children": []
}
```

### `frame` 最小结构

```json
{
  "type": "frame",
  "id": "frame-1",
  "name": "Console Panel",
  "width": 360,
  "height": 180,
  "children": []
}
```

### `text` 最小结构

```json
{
  "type": "text",
  "id": "text-1",
  "name": "Panel Title",
  "fill": "$--foreground",
  "content": "Run history",
  "fontFamily": "$--font-secondary",
  "fontSize": 16,
  "fontWeight": "500",
  "lineHeight": 1.5
}
```

### `rectangle` 最小结构

```json
{
  "type": "rectangle",
  "id": "rect-1",
  "name": "Signal Bar",
  "width": 240,
  "height": 2,
  "fill": "$--primary"
}
```

### `icon_font` 最小结构

```json
{
  "type": "icon_font",
  "id": "icon-1",
  "name": "Search Icon",
  "width": 16,
  "height": 16,
  "iconFontName": "search",
  "iconFontFamily": "lucide",
  "fill": "$--muted-foreground"
}
```

### `ref` 最小结构

```json
{
  "type": "ref",
  "id": "button-instance-1",
  "name": "Button/Secondary",
  "ref": "button-default",
  "descendants": {
    "button-label": {
      "content": "View logs"
    }
  }
}
```

## `variables` 的最小结构

颜色变量最小结构：

```json
{
  "--background": {
    "type": "color",
    "value": [
      { "value": "#111111", "theme": { "Mode": "Dark" } }
    ]
  }
}
```

数字变量最小结构：

```json
{
  "--radius-m": {
    "type": "number",
    "value": [
      { "value": 16, "theme": { "Mode": "Dark" } }
    ]
  }
}
```

字符串变量最小结构：

```json
{
  "--font-secondary": {
    "type": "string",
    "value": [
      { "value": "Geist", "theme": { "Mode": "Dark" } }
    ]
  }
}
```

## Lunaris 布局偏好

- 默认按深色产品界面或开发者工具页面组织，优先桌面端宽布局。
- 页面容器优先 `layout: "vertical"`，模块容器常见 `vertical`，局部工具条和筛选区常见 `horizontal`。
- 常用 `gap` 为 `12 / 16 / 20 / 24`，常用 `padding` 为 `[20,20,20,20]`、`[24,24,24,24]`、`[28,28,28,28]`、`[32,32,32,32]`。
- 首屏优先“标题区 + 信号标签 + 核心操作 + 产品预览面板”。
- 工作台页面优先“Sidebar + Content Panels + Table / Logs / Metrics”结构。
- 同一模块内部优先做清晰分区，不要在暗色背景上叠过多悬浮块。
- 发光感只点到为止，优先让描边、对比和间距承担层级表达。

## 可复用组件契约

如果某个结构会在两个以上模块重复出现，优先抽成 `reusable: true` 的基础组件，再通过 `ref` 实例化。

推荐基础组件：

- `Button/Default`
- `Button/Secondary`
- `Button/Outline`
- `Button/Ghost`
- `Icon Button/Default`
- `Input Group/Default`
- `Search Box/Default`
- `Select Group/Default`
- `Tabs`
- `Card`
- `Alert/Info`
- `Sidebar`
- `Data Table`
- `Dialog`
- `Label/Orange`

推荐实例化原则：

- 同一语义只保留一份基础骨架，状态变化优先用 `ref`
- Filled、Active、Warning、Error 这类状态不要重新造结构
- 改文案、图标、局部颜色、可见性时优先覆盖 descendants
- 控制台、日志、数据表、侧边导航只在产品壳层或工作台场景启用
- 官网首屏和营销收口不要默认套用 `Sidebar`、`Data Table`

## `ref` 复用逻辑

`ref` 的职责是“继承结构骨架 + 覆盖少量状态字段”，不是复制整个组件。

优先覆盖：

- `content`
- `fill`
- `stroke`
- `iconFontName`
- `iconFontFamily`
- `width` / `height`
- `enabled`

覆盖规则：

- 换按钮文案，只改内部 label 的 `content`
- 输入默认态切换到已填写态，优先改占位内容和文字颜色
- 搜索框有值时优先开启清除图标，不复制第二套新搜索栏
- Alert 的信息态、警告态、错误态共用同一内容骨架，只改 token 与图标
- Tabs 激活态只改底色、描边、文字色，不改命名主干
- `Sidebar`、`Tabs`、`Breadcrumb` 不能互相替代
- `Data Table`、`Card Grid`、`Alert` 不能因为视觉相近就混为一种组件

## 组件逻辑速记

### Button

- `Button/Default` 用于核心命令动作，如开始生成、运行、部署、连接
- `Button/Secondary` 用于并列补充动作，如查看日志、打开设置
- `Button/Outline` 用于筛选、工具条、列表级操作
- `Button/Ghost` 用于行内轻操作，不承担主 CTA
- `Icon Button/*` 只用于工具动作，图标语义不明确时要配 Tooltip
- `Button/Destructive` 只用于删除、重置、断开连接等危险动作

### Input / Search / Select

- `Input Group/*` 表示带标签的正式配置表单
- `Search Box/*` 只用于查询、过滤、命令查找，不替代资料录入表单
- `Select Group/*` 用于模型、环境、状态、工作区切换
- Filled 状态优先继承 Default，只改内容和状态色

### Tabs / Sidebar / Breadcrumb

- `Sidebar` 表示产品壳层导航
- `Tabs` 表示当前页面或模块内视图切换
- `Breadcrumb` 只表示层级路径
- 三者不能混用为同一种导航

### Card / Alert / Dialog

- `Card` 是日志块、能力卡、配置面板、预览窗的内容骨架
- `Alert/*` 只放短状态反馈、风险提示、系统通知
- `Dialog` 用于确认操作和聚焦设置，不承担长流程介绍
- 不要把 `Alert` 当长文卡片，也不要把 `Dialog` 当整页布局

### Data Table / Label / Progress

- `Data Table` 用于运行记录、任务队列、版本清单、模型评估
- `Pagination` 只用于列表翻页，不替代 Tabs
- `Label/*` 和 `Icon Label/*` 用于状态、版本、能力标签，不用于按钮
- `Progress` 用于任务进度和配额使用率，不承担主视觉装饰
