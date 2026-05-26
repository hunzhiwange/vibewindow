# Nitro 设计约束

## 输出硬规则

- 只输出 JSON，不输出解释、Markdown 代码块、注释、`null`、`undefined`、尾逗号。
- 根对象只保留：`version`、`theme`、`variables`、`children`。
- `version` 固定为 `"2.6"`。
- `theme` 默认固定为 `{ "Mode": "Light" }`。
- `children` 必须是数组。
- 页面骨架优先使用 `frame + text + ref`，避免为了装饰引入复杂图形。
- 颜色、字体、圆角优先引用 Nitro token，不要大面积硬编码颜色值。
- 默认生成企业官网 / B 端产品页 / 服务介绍页，不默认生成消费级营销感、超圆角、强情绪视觉。

## 允许元素与白名单

### 允许元素

- `frame`
- `text`
- `rectangle`
- `icon_font`
- `ref`

优先级：

1. 页面与模块骨架优先使用 `frame`
2. 标题、说明、指标优先使用 `text`
3. 分隔线、面板底色再补 `rectangle`
4. 语义图标才使用 `icon_font`
5. 有稳定复用价值时再使用 `reusable: true` 与 `ref`

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
  "width": 320,
  "height": 160,
  "children": []
}
```

### `text` 最小结构

```json
{
  "type": "text",
  "id": "text-1",
  "name": "Title",
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
  "width": 320,
  "height": 1,
  "fill": "$--border"
}
```

### `icon_font` 最小结构

```json
{
  "type": "icon_font",
  "id": "icon-1",
  "name": "Chevron Down",
  "width": 16,
  "height": 16,
  "iconFontName": "chevron-down",
  "iconFontFamily": "lucide",
  "fill": "$--muted-foreground"
}
```

### `ref` 最小结构

```json
{
  "type": "ref",
  "id": "button-instance-1",
  "name": "Button/Outline",
  "ref": "button-default",
  "descendants": {
    "button-label": {
      "content": "查看方案"
    }
  }
}
```

## variables 的最小结构

颜色变量最小结构：

```json
{
  "--primary": {
    "type": "color",
    "value": [
      { "value": "#0F5FFE", "theme": { "Mode": "Light" } },
      { "value": "#0F5FFE", "theme": { "Mode": "Dark" } }
    ]
  }
}
```

数字变量最小结构：

```json
{
  "--radius-none": {
    "type": "number",
    "value": [
      { "value": 0, "theme": { "Mode": "Light" } },
      { "value": 0, "theme": { "Mode": "Dark" } }
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
      { "value": "Roboto", "theme": { "Mode": "Light" } },
      { "value": "Roboto", "theme": { "Mode": "Dark" } }
    ]
  }
}
```

## Nitro 布局偏好

- 页面优先使用单栏垂直流，模块边界明确，信息分组稳定。
- 常用 `gap` 为 `8 / 12 / 16 / 24 / 32`。
- 常用 `padding` 为 `[16,16,16,16]`、`[24,24,24,24]`、`[32,32,32,32]`。
- 首屏优先“标题 + 说明 + 主次操作 + 证据块 / 视觉块”。
- 中段优先“能力矩阵 / 方案分组 / 指标卡 / 流程说明 / 对比表”。
- 表单、表格、侧边栏、卡片优先直角或极小圆角，默认使用 `$--radius-none`，悬浮提示可用 `$--radius-xs`。
- 模块内部优先“标题区 + 内容区 + 动作区”三段结构，避免漂浮装饰块抢占信息层级。
- 桌面端更偏稳定网格与等宽列，移动端再收窄宽度，不要直接复制营销长图式堆叠。

## 可复用组件契约

如果某个结构会在两个以上模块重复出现，就优先抽成 `reusable: true` 的基础组件，再通过 `ref` 做实例化。

- 基础组件骨架与变体继承关系看 `nitro/components`

推荐基础组件：

- `Button/Default`
- `Button/Secondary`
- `Button/Outline`
- `Button/Ghost`
- `Input Group/Default`
- `Select Group/Default`
- `Textarea Group/Default`
- `Checkbox/Checked`
- `Radio/Selected`
- `Switch/Checked`
- `Tabs`
- `Card`
- `Alert/Info`
- `Sidebar`
- `Table`
- `Data Table`
- `Pagination`
- `Accordion/Open`

推荐实例化原则：

- 同一语义只保留一份基础骨架
- 状态变化优先通过 `ref + descendants` 表达
- 填写态、选中态、激活态优先覆盖颜色、文案、显隐，不复制新结构
- 紧凑版控件优先隐藏标签或次级说明，不改主命名干线
- 大尺寸按钮只表达尺寸升级，不改变按钮语义

## ref 复用逻辑

`ref` 的职责是“继承 Nitro 组件骨架 + 覆盖局部状态”，不是重新造一套新组件。

优先覆盖：

- `content`
- `fill`
- `stroke`
- `iconFontName`
- `iconFontFamily`
- `width` / `height`
- `padding`
- `enabled`

覆盖规则：

- 输入框默认态与填写态，只改值文本与文字颜色
- Select 默认态与已选态，只改触发器文本内容与颜色
- Checkbox / Radio / Switch 的状态切换，只改选中标记、描边、背景与显隐
- Tabs / Pagination / Sidebar 的激活态，只改前景色、描边、底色，不改整体骨架
- Card 的 Plain / Action / Image 只改 Header / Content / Actions 的内容编排
- Alert 的 Success / Warning / Error 继承 `Alert/Info` 骨架，只切换状态色与图标
- 需要删掉副标题、图标、描述时，优先 `enabled: false`

## 组件逻辑速记

### Button

- `Button/Default` 用于首屏主 CTA、表单提交、方案卡收口
- `Button/Secondary` 用于主按钮旁的备选动作
- `Button/Outline` 用于低强调但仍需明确边界的操作
- `Button/Ghost` 用于工具栏、筛选条、分页前后切换
- `Icon Button/*` 只用于工具动作，不承担核心转化
- `Button/Large/*` 只用于首屏或模块收口，不要塞进密集表格行内

### Input / Select / Textarea

- `Group/*` 表示正式表单区，含标签与输入壳
- `Input/*`、`Select/*` 表示紧凑版控件，适合工具栏、筛选条、表格头部
- `Default` 表示占位态，`Filled` 表示已填写或已选中
- `Input OTP Group/*` 只用于验证码，不要混到联系表单或服务咨询表单

### Checkbox / Radio / Switch

- Checkbox 表示多选
- Radio 表示互斥单选
- Switch 表示即时开关
- `Description/*` 适合方案选择、权限选项、带解释的设置项
- 不要用 Switch 伪装单选，也不要用 Checkbox 伪装互斥方案

### Card / Table / Sidebar

- `Card Plain` 适合指标、摘要、能力说明
- `Card Action` 适合服务方案、套餐卡、带 CTA 的能力块
- `Card Image` 适合案例、产品卡、服务图文模块
- `Table` / `Data Table` 适合后台、方案对比、结构化记录
- `Sidebar` 适合工作台、后台、产品台，不适合官网首屏和轻营销落地页

### Tabs / Pagination / Breadcrumb / Accordion

- Tabs 只做模块内切换
- Pagination 只做列表翻页
- Breadcrumb 只做层级路径提示
- Accordion 只做 FAQ、政策、细则折叠
- 这四类组件不能互相替代

### Alert / Tooltip / Label

- Alert 用于状态反馈、系统提示、结果确认
- Tooltip 只放一句短提示，不放大段说明
- Label 用于状态标签、分类标签、轻量徽标
- 不要把 Alert 当卡片主体，也不要把 Tooltip 当详情容器

### Nitro 风格边界

- 不要做 Halo 的胶囊按钮、柔和圆角输入框、轻转化情绪化视觉
- 不要做 Lunaris 的高对比暗黑未来感和重霓虹表达
- 默认保持理性、规整、企业级结构稳定感
