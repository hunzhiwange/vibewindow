# Halo 设计约束

## 输出硬规则

- 只输出 JSON，不输出解释、Markdown 代码块、注释、`null`、`undefined`、尾逗号。
- 根对象只保留：`version`、`theme`、`variables`、`children`。
- `version` 固定为 `"2.6"`。
- `theme` 固定为 `{ "Mode": "Light" }`。
- `children` 必须是数组。

## 允许元素22223322

- `frame`
- `text`
- `rectangle`
- `icon_font`
- `ref`

优先级：

1. 页面骨架优先只用 `frame + text`
2. 分隔或底色再补 `rectangle`
3. 需要图标时才用 `icon_font`
4. 有明确复用价值时再用 `reusable: true` 与 `ref`

## 常用字段白名单

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

## 最小 JSON 结构

下面这些结构不是完整页面，只是给 AI 的“最小合法骨架”，目的是让它知道每种元素至少该长什么样。

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
  "name": "Container",
  "width": 320,
  "height": 120,
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
  "content": "标题",
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
  "name": "Plus Icon",
  "width": 16,
  "height": 16,
  "iconFontName": "plus",
  "iconFontFamily": "lucide",
  "fill": "$--primary"
}
```

### `ref` 最小结构

```json
{
  "type": "ref",
  "id": "button-instance-1",
  "name": "Button/Secondary",
  "ref": "button-base",
  "descendants": {
    "button-label": {
      "content": "Secondary Action"
    }
  }
}
```

### `variables` 最小结构

颜色变量最小结构：

```json
{
  "--secondary": {
    "type": "color",
    "value": [
      { "value": "#D9D9DB", "theme": { "Mode": "Light" } },
      { "value": "#403F51", "theme": { "Mode": "Dark" } }
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

## 推荐结构

一个网站只生成一个 `.pen` 文件，推荐结构：

1. 根 `children` 放项目容器 `frame`
2. 项目容器内放多个页面 `frame`
3. 每个页面内再放多个模块 `frame`
4. 模块内部使用标题、描述、操作区、卡片区等语义容器

最小骨架：

```json
{
  "version": "2.6",
  "theme": { "Mode": "Light" },
  "variables": {},
  "children": [
    {
      "type": "frame",
      "id": "project-root",
      "name": "AI Project Pen",
      "children": [
        {
          "type": "frame",
          "id": "design-page-0",
          "name": "首页",
          "children": []
        }
      ]
    }
  ]
}
```

## 页面与模块写法

- 页面名称必须接近真实导航，如“首页”“定价”“案例”“联系”。
- 模块名称必须接近真实内容块，如“品牌首屏”“卖点卡片”“客户证言”“联系收口”。
- 模块必须可单独替换，不依赖整站上下文才能成立。
- 不要使用“模块一”“步骤二”“容器 A”这类无语义命名。

## Halo 结构偏好

- 页面容器优先 `layout: "vertical"`。
- 常用 `gap` 为 `12 / 16 / 20 / 24`。
- 常用 `padding` 为 `[16,16,16,16]`、`[20,20,20,20]`、`[24,24,24,24]`。
- 卡片、按钮、输入框、标签优先圆角，常用 `$--radius-m` 或 `$--radius-pill`。
- 模块内部优先使用“文案区 + 操作区 + 卡片区”的清晰层级。

## 可复用组件契约

如果某个结构会在两个以上模块重复出现，就优先抽成 `reusable: true` 的基础组件，再通过 `ref` 做实例化。

- 基础组件骨架与变体继承关系看 `halo/components`

推荐基础组件：

- `Button/Default`
- `Button/Secondary`
- `Icon Button/Default`
- `Input Group/Default`
- `Textarea Group/Default`
- `Checkbox/Checked`
- `Checkbox/Unchecked`
- `Radio/Selected`
- `Radio/Unselected`
- `Switch/Checked`
- `Switch/Unchecked`
- `Tabs`
- `Card`
- `Alert/Info`
- `Pagination`

推荐实例化原则：

- 同一语义只保留一份基础骨架
- 状态变化优先通过 `ref + descendants` 表达
- 不因为文案不同就复制一套新组件
- 不因为颜色变化就改组件名字，优先覆盖 `fill`

## `ref` 复用逻辑

`ref` 的职责是“继承骨架 + 覆盖少量字段”，不是重新造组件。

优先覆盖：

- `content`
- `fill`
- `stroke`
- `iconFontName`
- `iconFontFamily`
- `width` / `height`
- `enabled`

覆盖规则：

- 换按钮文案，只改内部 `text.content`
- 换图标，只改内部 `icon_font`
- 去掉可选图标或副标题，优先 `enabled: false`
- 从默认态切换到激活态，优先改颜色、描边、阴影、文案色
- 只有状态差异时，必须保留相同命名主干，如 `Button/*`、`Checkbox/*`

## 组件逻辑速记

### Button

- 主按钮用于首屏 CTA、表单提交、重点转化
- 次按钮用于取消、返回、补充动作
- Outline / Ghost 用于低强调操作
- Icon Button 只用于纯工具动作，不承担核心 CTA

### Input / Textarea / Select

- `Group/*` 表示带标签的完整输入区
- 无 `Group` 表示紧凑版控件
- `Default` 表示占位态
- `Filled` 表示已填写态
- 输入类优先复用同一骨架，只切换占位文案和文字颜色

### Checkbox / Radio / Switch

- Checkbox 表示多选
- Radio 表示互斥单选
- Switch 表示即时开关
- `Description/*` 表示该选项还需要标题与辅助说明

### Tabs / Pagination / Breadcrumb

- Tabs 只做模块内切换
- Pagination 只做列表翻页
- Breadcrumb 只做路径层级提示
- 三者不能互相替代

### Card / Alert / Tooltip

- Card 是内容承载骨架，默认按 Header / Content / Actions 三段理解
- Alert 用于状态反馈，不用于长文内容
- Tooltip 只放一句短提示，不放复杂说明

### Sidebar / List / Table

- 这些组件偏后台、工作台、管理视图
- 如果用户要的是官网首页、活动页、转化页，优先少用
- 只有用户明确要求复杂导航或数据视图时再启用

## Token 规则

- 颜色、字体、圆角优先用 token，不写死大量颜色值。
- 常用 token：
  - `$--background`
  - `$--foreground`
  - `$--card`
  - `$--card-foreground`
  - `$--primary`
  - `$--primary-foreground`
  - `$--secondary`
  - `$--muted-foreground`
  - `$--border`
  - `$--sidebar`
  - `$--font-primary`
  - `$--font-secondary`
  - `$--radius-m`
  - `$--radius-pill`

## 文案与内容限制

- 可以写占位文案，但要像真实官网模块。
- 不要虚构品牌名、价格、客户名单、案例数据、统计指标。
- 不要把模块输出成完整网站项目。
- 不要堆太多绝对定位，优先 auto layout。