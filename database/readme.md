# 数据库

- 使用postgresql

## 常用的数据结构
- 作品
```

work:{
"title": "作品标题",
"material": "作品素材",
"description": "作品描述",
"created_at": "创建时间",
}
```

## 与前端的同步逻辑
- API 需要按照“节点”语义返回数据，所有返回体在前端会被视为 `node`，再映射到 Overview / Present / Detail 三栏。
- `directories` 接口仍提供目录节点，但命名上需保持与前端一致，避免再混用 `directory` / `node`。
- Detail 栏除了渲染文件，也会把目录节点以可点击条目呈现，因此接口要保证每个节点具备 `raw_filename`、`path` 等基础字段，供前端生成节点描述。
- 当移动端通过 Detail 列表导航目录时，会直接命中这些 API，因此保持节点字段稳定是跨端一致性的关键。
