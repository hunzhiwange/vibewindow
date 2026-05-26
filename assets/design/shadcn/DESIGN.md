# Shadcn 设计约束

## 输出硬规则

- 只输出 JSON，不输出解释、Markdown 代码块、注释、`null`、`undefined`、尾逗号。
- 根对象只保留：`version`、`theme`、`variables`、`children`。
- `version` 固定为 `"2.6"`。
- 默认 `theme` 为 `{ "Mode": "Light" }`。
- `children` 必须是数组。
- 页面优先依赖 token、布局和排版建立层级，不依赖重装饰、营销型背景块或情绪化渐变。
- 没有明确复用价值时，不新增 `reusable: true`；没有明确继承关系时，不使用 `ref`。

## 允许元素与白名单

允许元素：

- `frame`
- `text`
- `rectangle`
- `icon_font`
- `ref`

优先级：

1. 页面骨架优先 `frame + text`
2. 分隔、弱底色、输入壳体再补 `rectangle`
3. 图标只在语义明确时使用 `icon_font`
4. 同一语义出现两次以上，再抽成 `reusable: true` 与 `ref`

### `frame`

- `type` `id` `name` `x` `y` `width` `height` `fill`
- `children` `layout` `gap` `padding`
- `justifyContent` `alignItems`
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
  "theme": { "Mode": "Light" },
  "variables": {},
  "children": []
}
```

### `frame` 最小结构

```json
{
  "type": "frame",
  "id": "frame-1",
  "name": "Section",
  "width": 360,
  "height": 160,
  "children": []
}
```

### `text` 最小结构

```json
{
  "type": "text",
  "id": "text-1",
  "name": "Heading",
  "fill": "$--foreground",
  "content": "模块标题",
  "fontFamily": "$--font-primary",
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
  "name": "Divider",
  "width": 360,
  "height": 1,
  "fill": "$--border"
}
```

### `icon_font` 最小结构

```json
{
  "type": "icon_font",
  "id": "icon-1",
  "name": "Chevron Right",
  "width": 16,
  "height": 16,
  "iconFontName": "chevron-right",
  "iconFontFamily": "lucide",
  "fill": "$--foreground"
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
      "content": "Secondary Action"
    }
  }
}
```

## `variables` 的最小结构

颜色变量最小结构：

```json
{
  "--border": {
    "type": "color",
    "value": [
      { "value": "#e5e5e5", "theme": { "Mode": "Light" } },
      { "value": "#ffffff1a", "theme": { "Mode": "Dark" } }
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
      { "value": 6, "theme": { "Mode": "Light" } },
      { "value": 6, "theme": { "Mode": "Dark" } }
    ]
  }
}
```

字符串变量最小结构：

```json
{
  "--font-primary": {
    "type": "string",
    "value": [
      { "value": "Inter", "theme": { "Mode": "Light" } },
      { "value": "Inter", "theme": { "Mode": "Dark" } }
    ]
  }
}
```

## Shadcn 布局偏好

- 页面优先单栏垂直流，再在模块内部切 2 列或 3 列。
- 容器节奏优先 `gap: 8 / 12 / 16 / 24 / 32`。
- 常用 `padding` 优先 `[16,16,16,16]`、`[24,24,24,24]`、`[32,32,32,32]`。
- 卡片、按钮、输入框优先中等圆角，常用 `$--radius-s`、`$--radius-m`、`$--radius-l`。
- 大多数内容模块优先“标题区 + 描述区 + 内容区 + 操作区”。
- 视觉重点依赖标题层级、容器边框、留白和轻阴影，不依赖重色块冲击。
- 官网首屏可以有 CTA，但应保持克制，不做 Halo 式强转化压迫感。

## 可复用组件契约

如果某个组件骨架会在两个以上模块重复出现，就优先抽成 `reusable: true` 的基础组件，再通过 `ref` 实例化。

- 基础骨架保持中性命名，不把颜色直接写进语义
- 状态变化优先走 `ref + descendants`
- 文案变化、图标变化、显隐变化都优先覆盖，不复制整套结构
- 尺寸差异只在确实改变命中区域或层级时再拆大号变体

推荐基础组件：

- `Button/Default`
- `Button/Secondary`
- `Button/Outline`
- `Button/Ghost`
- `Icon Button/Default`
- `Badge/Default`
- `Input Group/Default`
- `Textarea Group/Default`
- `Select Group/Default`
- `Checkbox/Checked`
- `Radio/Selected`
- `Switch/Checked`
- `Card`
- `Alert/Default`
- `Tabs`
- `Pagination`
- `Sidebar`

## `ref` 复用逻辑

`ref` 的职责是“继承骨架 + 覆盖少量字段”，不是复制一份相似组件。

优先覆盖：

- `content`
- `fill`
- `stroke`
- `width` / `height`
- `iconFontName`
- `enabled`

优先复用的状态切换：

- `Button/Default` → `Button/Secondary` / `Button/Outline` / `Button/Ghost`
- `Input Group/Default` → `Input Group/Filled`
- `Checkbox/Checked` ↔ `Checkbox/Unchecked`
- `Radio/Selected` ↔ `Radio/Unselected`
- `Switch/Checked` ↔ `Switch/Unchecked`
- `Tab Item/Active` ↔ `Tab Item/Inactive`
- `Sidebar Item/Active` ↔ `Sidebar Item/Default`

覆盖规则：

- 换按钮文案，只改内部 label 的 `content`
- 换图标，只改内部 `icon_font`
- 占位态切已填写态，优先改输入文字颜色与内容
- 非激活态切激活态，优先改背景、文字色、描边或轻阴影
- 取消图标、说明、尾部动作时，优先 `enabled: false`

## 组件逻辑速记

### Button

- `Default` 用于模块主动作、提交、继续、确认
- `Secondary` 用于补充动作、返回、取消、备用入口
- `Outline` 用于需要边界但不抢主次的操作
- `Ghost` 用于列表工具条、轻量行内操作
- `Icon Button` 只用于纯工具行为，不替代唯一主 CTA

### Input / Textarea / Select

- `Group/*` 表示带标签的完整输入区
- `Default` 表示占位态，`Filled` 表示已有值
- `Input` 适合单行输入，`Textarea` 适合中短说明
- `Select` 只用于离散选项，不与自由输入混用
- 表单骨架优先统一输入高、圆角、描边和文字层级

### Checkbox / Radio / Switch

- Checkbox 表示多选或列表勾选
- Radio 表示互斥单选
- Switch 表示即时开关，不用于提交后才生效的确认场景
- 同一组选项不要混用 Checkbox 与 Radio

### Badge / Alert

- Badge 只做状态标签、分类标签、轻提示
- Alert 用于状态说明、风险提醒、系统反馈
- Badge 不能代替按钮，Alert 不能代替正文容器

### Card

- Card 是内容承载骨架，默认理解为 Header / Content / Actions
- 说明类、能力类、价格类、摘要类都可复用 Card
- 如果只是纯段落信息，不必强行卡片化

### Tabs / Pagination / Breadcrumb

- Tabs 只做模块内内容切换
- Pagination 只做分页
- Breadcrumb 只做路径提示
- 三者不能互相替代，也不要拿 Tabs 做主导航

### Sidebar / Table

- Sidebar 更适合工作台、后台、文档台，不是默认官网首页导航
- Table 适合数据列表、管理信息，不适合营销首屏
- 如果页面目标是产品官网或内容站，优先顶部导航和卡片流

### Tooltip / Dropdown / Accordion

- Tooltip 只放一句短解释
- Dropdown 用于上下文选项集合，不承载长信息块
- Accordion 更适合 FAQ、设置说明折叠，不要替代页面主结构

## Token 规则

- 颜色、字体、圆角优先走 token，不大量写死色值
- 常用 token：
  - `$--background`
  - `$--foreground`
  - `$--card`
  - `$--card-foreground`
  - `$--primary`
  - `$--primary-foreground`
  - `$--secondary`
  - `$--muted`
  - `$--muted-foreground`
  - `$--accent`
  - `$--border`
  - `$--input`
  - `$--sidebar`
  - `$--sidebar-foreground`
  - `$--radius-s`
  - `$--radius-m`
  - `$--radius-l`
  - `$--radius-pill`
  - `$--font-primary`
  - `$--font-secondary`

## 文案与内容限制

- 可以写真实感占位文案，但不要虚构品牌背书、价格、客户名单、统计数字。
- 不要把页面写成强营销落地页话术。
- 不要堆满装饰图形、超大渐变、玻璃拟态或未来科技光效。
- 不要把后台控件硬塞进官网首屏，也不要把官网 CTA 逻辑搬进数据表格页。
