# CSS 样式组织规范

## 目录结构

```
src/
├── styles/
│   ├── base.css              # 基础样式和浏览器重置
│   ├── components.css        # 通用组件样式
│   ├── layout.css            # 布局相关样式
│   ├── utils.css             # 工具类样式
│   └── element-plus.css      # Element Plus 样式覆盖
```

## 各文件说明

### base.css
- 包含项目的基础样式和浏览器重置
- 定义全局字体、颜色变量等

### layout.css
- 包含页面布局相关的样式
- 如容器、页面头部、主要内容区域等

### components.css
- 包含通用UI组件的样式
- 如卡片、按钮、表单元素等

### utils.css
- 包含常用的工具类样式
- 如间距、文本对齐、显示隐藏等

### element-plus.css
- 包含Element Plus组件的样式覆盖
- 使用深度选择器 `:deep()` 正确修改内部样式

## 使用规范

1. 所有样式应按照功能分类放入对应的CSS文件中
2. 避免在组件中编写重复的样式代码
3. 使用语义化的类名，便于理解和维护
4. 类名使用小写字母，单词间以连字符分隔
5. 避免使用ID选择器和!important