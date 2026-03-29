# Contributing Guide / 贡献指南

[中文](./CONTRIBUTING.md) | [English](./CONTRIBUTING_ZH.md)

---

感谢您考虑为 SMA-OS 做出贡献！

## 📋 目录

- [行为准则](#行为准则)
- [如何贡献](#如何贡献)
- [开发流程](#开发流程)
- [代码规范](#代码规范)
- [提交信息](#提交信息)
- [Pull Request 流程](#pull-request-流程)

## 行为准则

请尊重所有贡献者，保持友好和建设性的讨论氛围。

## 如何贡献

### 报告 Bug

1. 在 [Issues](https://github.com/LING71671/SMA-OS/issues) 中搜索是否已有相关问题
2. 如果没有，创建新 Issue，包含：
   - 清晰的标题
   - 复现步骤
   - 预期行为与实际行为
   - 环境信息（操作系统、版本等）

### 提交功能建议

1. 在 Issues 中描述您希望添加的功能
2. 说明该功能的使用场景和价值
3. 等待维护者反馈后再开始实现

### 提交代码

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/your-feature`)
3. 进行修改并添加测试
4. 提交 Pull Request

## 开发流程

### 环境设置

```bash
# 克隆仓库
git clone https://github.com/LING71671/SMA-OS.git
cd SMA-OS

# 启动基础设施
docker-compose up -d

# 安装依赖
cd control-plane && cargo build
cd ../memory-bus && go mod download
cd ../orchestration && go mod download
cd ../observability-ui/web-dashboard && npm install
```

### 运行测试

```bash
# Go 测试
cd memory-bus && go test -v ./...
cd ../orchestration && go test -v ./...

# Rust 测试
cd control-plane && cargo test

# 前端测试
cd observability-ui/web-dashboard && npm run lint
```

## 代码规范

### Rust

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 所有公共 API 必须有文档注释 (`///`)
- 错误处理使用 `Result<T, Error>`，不要使用 `unwrap()`

### Go

- 使用 `gofmt` 格式化代码
- 使用 `golangci-lint` 检查代码质量
- 错误必须显式处理，不要忽略
- 导出函数必须有注释

### TypeScript

- 使用 `npm run lint` 检查代码
- 使用 TypeScript 严格模式
- 组件使用 PascalCase 命名

## 提交信息

提交信息格式：

```
<type>: <subject>

<body>

<footer>
```

### Type 类型

- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `test`: 测试相关
- `chore`: 构建/工具相关

### 示例

```
feat: 添加 DAG 执行超时配置

- 支持全局超时设置
- 支持单个任务超时配置
- 超时后自动清理资源

Closes #123
```

## Pull Request 流程

1. **确保测试通过**: 所有测试必须通过才能合并
2. **更新文档**: 如果有 API 变更，更新相关文档
3. **一个 PR 一个功能**: 避免在一个 PR 中混合多个不相关的修改
4. **等待 Review**: 维护者会尽快审核您的 PR

### PR 检查清单

- [ ] 代码通过所有测试
- [ ] 新功能有对应的测试
- [ ] 文档已更新
- [ ] 提交信息格式正确
- [ ] 没有合并冲突

## 代码审查

所有提交都需要经过代码审查。审查重点：

1. 代码质量和可读性
2. 测试覆盖率
3. 文档完整性
4. 是否符合项目架构

## 问题？

如果您有任何问题，可以：

1. 在 [Issues](https://github.com/LING71671/SMA-OS/issues) 中提问
2. 查看 [部署文档](../ops/DEPLOYMENT.md)

---

感谢您的贡献！
