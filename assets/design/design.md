# `.json` 设计文档结构约束

本文件描述 AI 生成设计文档时必须遵循的基础约束。

目标不是覆盖全部 schema，而是给页面规划与模块生成提供稳定的最小契约。

---

## 根结构

```json
{
  "version": "2.6",
  "children": [],
  "variables": {},
  "theme": { "Mode": "Light" }
}
```

## AI 专用白名单字段版

仅允许输出以下字段，超出字段一律删除。字段名区分大小写。

### 1) 根对象白名单

- `version`
- `theme`
- `variables`
- `children`

根对象必填：

- `version` 必须是 `"2.6"`
- `children` 必须是数组

### 2) `theme` 白名单

- `Mode`

`Mode` 仅允许：

- `Light`
- `Dark`

### 3) `variables` 白名单

变量定义仅允许：

- `type`
- `value`

`value` 数组项仅允许：

- `value`
- `theme`

`theme` 仅允许：

- `Mode`

### 4) 元素通用白名单

所有元素都只允许以下通用字段（按需出现）：

- `type`
- `id`
- `name`
- `x`
- `y`
- `width`
- `height`
- `fill`
- `children`
- `layout`
- `gap`
- `padding`
- `stroke`
- `cornerRadius`
- `clip`
- `reusable`
- `content`

### 5) 按类型字段白名单

`frame` 允许字段：

- `type` `id` `name` `x` `y` `width` `height` `fill` `children` `layout` `gap` `padding` `stroke` `cornerRadius` `clip` `reusable`

`text` 允许字段：

- `type` `id` `name` `x` `y` `width` `height` `fill` `content` `fontFamily` `fontSize` `fontWeight` `lineHeight` `textAlign` `textGrowth`

`rectangle` 允许字段：

- `type` `id` `name` `x` `y` `width` `height` `fill` `stroke` `cornerRadius`

`icon_font` 允许字段：

- `type` `id` `name` `x` `y` `width` `height` `fill` `iconFontName` `iconFontFamily`

`ref` 允许字段：

- `type` `id` `name` `x` `y` `width` `height` `ref` `descendants`

### 6) `stroke` 白名单

- `align`
- `thickness`
- `fill`

### 7) `descendants` 白名单

- 顶层是对象（key 为被覆盖节点 id）
- 每个覆盖对象仅允许与被覆盖类型对应的白名单字段
- 禁止在 `descendants` 中创建新节点或新增 `type`、`id`

### 8) 生成硬规则

- 只输出 JSON，不输出解释文字。
- 不能出现 `null`、`undefined`、注释、尾逗号。
- `type` 仅允许：`frame` `text` `rectangle` `icon_font` `ref`。
- 任何未在白名单中的字段必须删除。
- `ref` 节点必须包含 `ref`，且值必须指向已存在的 `reusable: true` 节点 `id`。

说明：

- `version` 固定为 `2.6`。
- `children` 是页面或模块节点集合。
- `variables` 保存 design token。
- `theme.Mode` 通常为 `Light` 或 `Dark`，与所选主题保持一致。

---

## 项目级约束

- 一个网站项目只生成一个 `.json` 文件。
- 同一个项目中的多个页面放在同一个根文档里。
- 页面优先使用顶层 `frame` 表示。
- 每个页面内部再用 `frame` 组织模块。

推荐结构：

```json
{
  "version": "2.6",
  "children": [
    {
      "type": "frame",
      "id": "project-pen-root",
      "name": "AI Project Json",
      "children": [
        {
          "type": "frame",
          "id": "design-page-0",
          "name": "首页"
        }
      ]
    }
  ]
}
```

---

## 页面约束

页面是顶层或项目容器内的 `frame`。

推荐字段：

```json
{
  "type": "frame",
  "id": "design-page-0",
  "name": "首页",
  "theme": { "Mode": "Light" },
  "width": 420,
  "height": 900,
  "fill": "$--background",
  "children": []
}
```

规则：

- 页面名称必须接近真实网站导航。
- 页面应包含目标说明、状态提示和模块容器。
- 页面容器内的模块要可被单独替换、单独生成、单独汇总。

---

## 模块约束

模块通常也是 `frame`，用于承载某一段页面内容。

推荐字段：

```json
{
  "type": "frame",
  "id": "page-0-module-0",
  "name": "品牌首屏",
  "context": "用于展示价值主张、辅助文案、CTA 与视觉焦点",
  "width": 372,
  "height": 148,
  "fill": "$--card",
  "stroke": {
    "align": "inside",
    "thickness": 1,
    "fill": "$--border"
  },
  "children": []
}
```

规则：

- 模块必须能作为局部片段导入页面容器。
- 模块内部优先使用容器、标题、描述、操作区等明确层级。
- 模块输出不要依赖整个站点上下文才能成立。

---

## 常用元素类型

| type | 用途 |
|------|------|
| `frame` | 页面、容器、卡片、模块 |
| `text` | 标题、正文、标签、说明 |
| `rectangle` | 背景块、分隔、装饰 |
| `icon_font` | 图标 |
| `ref` | 引用可复用组件 |

页面规划阶段一般只需要 `frame` 与 `text`。
模块细化阶段可以进一步使用 `ref`、`icon_font`、`rectangle`。

---

## 布局建议

优先使用 `frame + auto layout`。

常用写法：

```json
{
  "layout": "vertical",
  "gap": 12,
  "padding": [16, 16, 16, 16]
}
```

```json
{
  "layout": "horizontal",
  "justifyContent": "space_between",
  "alignItems": "center"
}
```

原则：

- 页面骨架阶段先保证层级清楚。
- 模块生成阶段再补充局部视觉与子结构。
- 除非必要，不要一开始就使用复杂绝对定位。

---

## Token 约束

颜色、字体、圆角优先引用变量，不要写死。

常用 token：

- `--background`
- `--foreground`
- `--card`
- `--card-foreground`
- `--primary`
- `--primary-foreground`
- `--secondary`
- `--secondary-foreground`
- `--muted-foreground`
- `--border`
- `--sidebar`
- `--font-primary`
- `--font-secondary`

引用方式：

```json
{
  "fill": "$--card"
}
```

---

## 主题约束

- `shadcn`、`nitro`、`halo` 默认使用 `Light`。
- `lunaris` 默认使用 `Dark`。
- 变量和主题模式要与对应主题文档保持一致。

---

## 命名约束

- 页面用真实页面名，如“首页”“定价页”“案例页”“帮助中心”。
- 模块用真实内容块名，如“品牌首屏”“套餐矩阵”“客户证言”“商品列表”。
- 不要使用“步骤一”“模块一”“容器 A”这种缺乏语义的命名。

---

## 生成时禁止事项

- 不要输出 Markdown 代码块。
- 不要输出解释文本。
- 不要虚构品牌、价格、客户名单或统计指标。
- 不要忽略主题 token 直接写大量硬编码颜色。
- 不要把每个模块生成成一个完整网站项目。

---

## 关联资产

- `prompt.md`: 提示词流程与输出格式。
- `presets.md`: 预设页面骨架。
- `theme-*.md`: 主题 token 与风格说明。
- `*.json`: 对应主题 design system 样例。

## 最小可用 Demo

```json
{
  "version": "2.6",
  "theme": { "Mode": "Light" },
  "variables": {
    "--background": {
      "type": "color",
      "value": [{ "value": "#fafafa", "theme": { "Mode": "Light" } }]
    },
    "--foreground": {
      "type": "color",
      "value": [{ "value": "#18181b", "theme": { "Mode": "Light" } }]
    },
    "--card": {
      "type": "color",
      "value": [{ "value": "#ffffff", "theme": { "Mode": "Light" } }]
    },
    "--muted-foreground": {
      "type": "color",
      "value": [{ "value": "#71717a", "theme": { "Mode": "Light" } }]
    },
    "--primary": {
      "type": "color",
      "value": [{ "value": "#7c3aed", "theme": { "Mode": "Light" } }]
    },
    "--primary-foreground": {
      "type": "color",
      "value": [{ "value": "#fafafa", "theme": { "Mode": "Light" } }]
    },
    "--border": {
      "type": "color",
      "value": [{ "value": "#e4e4e7", "theme": { "Mode": "Light" } }]
    },
    "--radius-m": {
      "type": "number",
      "value": [{ "value": 6, "theme": { "Mode": "Light" } }]
    },
    "--radius-pill": {
      "type": "number",
      "value": [{ "value": 9999, "theme": { "Mode": "Light" } }]
    },
    "--font-primary": {
      "type": "string",
      "value": [{ "value": "Inter", "theme": { "Mode": "Light" } }]
    },
    "--font-secondary": {
      "type": "string",
      "value": [{ "value": "Inter", "theme": { "Mode": "Light" } }]
    }
  },
  "children": [
    {
      "type": "frame",
      "id": "project-root",
      "name": "Halo Minimal Demo",
      "x": 0,
      "y": 0,
      "clip": true,
      "width": 420,
      "height": 900,
      "fill": "$--background",
      "layout": "vertical",
      "gap": 12,
      "padding": [16, 16, 16, 16],
      "children": [
        {
          "type": "text",
          "id": "title-1",
          "name": "Title",
          "fill": "$--foreground",
          "textGrowth": "fixed-width",
          "width": "fill_container",
          "content": "Halo 最小可用 Demo",
          "lineHeight": 1.5,
          "fontFamily": "$--font-primary",
          "fontSize": 18,
          "fontWeight": "500",
          "textAlign": "left"
        },
        {
          "type": "rectangle",
          "id": "rect-1",
          "name": "Divider",
          "width": "fill_container",
          "height": 1,
          "fill": "$--border"
        },
        {
          "type": "icon_font",
          "id": "icon-1",
          "name": "Plus Icon",
          "width": 16,
          "height": 16,
          "iconFontName": "plus",
          "iconFontFamily": "lucide",
          "fill": "$--primary"
        },
        {
          "type": "frame",
          "id": "card-base",
          "name": "Card/Base",
          "reusable": true,
          "width": "fill_container",
          "height": 120,
          "fill": "$--card",
          "cornerRadius": "$--radius-m",
          "stroke": {
            "align": "inside",
            "thickness": 1,
            "fill": "$--border"
          },
          "layout": "vertical",
          "gap": 8,
          "padding": [12, 12, 12, 12],
          "children": [
            {
              "type": "text",
              "id": "card-title",
              "name": "Card Title",
              "fill": "$--foreground",
              "content": "默认标题",
              "lineHeight": 1.4,
              "fontFamily": "$--font-secondary",
              "fontSize": 14,
              "fontWeight": "500"
            },
            {
              "type": "text",
              "id": "card-desc",
              "name": "Card Desc",
              "fill": "$--muted-foreground",
              "content": "默认描述",
              "lineHeight": 1.4,
              "fontFamily": "$--font-secondary",
              "fontSize": 12,
              "fontWeight": "normal"
            }
          ]
        },
        {
          "type": "ref",
          "id": "card-instance",
          "name": "Card/Instance",
          "ref": "card-base",
          "descendants": {
            "card-title": {
              "content": "实例标题（ref override）"
            },
            "card-desc": {
              "content": "实例描述（可安全覆盖）"
            }
          }
        }
      ]
    }
  ]
}
```
