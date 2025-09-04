# Redis 集成测试使用指南

## 📋 测试概述

本项目提供了完整的 Redis 集成测试实例 (`examples/test_redis.rs`)，覆盖了 **功能性测试、性能测试、并发测试、错误处理测试** 四个主要维度，确保 Redis 模块的可靠性、性能、易用性和兼容性。

## 🏗️ 测试结构

### 测试覆盖的功能模块

1. **基础连接测试** - 验证 Redis 连接建立和配置
2. **配置管理测试** - 验证 RedisConfig 的各种功能
3. **基本操作测试** - GET/SET/EXISTS 等核心操作
4. **数据结构测试** - 列表操作 (LPUSH/RPOP) 和哈希操作 (HSET/HGET)
5. **错误处理测试** - 无效连接、配置验证等错误场景
6. **性能测试** - 连接建立时间、操作响应时间
7. **并发测试** - 多连接并发、多操作并发
8. **健康检查测试** - 连接稳定性和统计信息

### 测试维度分析

| 测试维度 | 覆盖内容 | 测试数量 |
|---------|----------|---------|
| **功能性测试** | 连接、配置、基本操作、数据结构操作 | 7个 |
| **性能测试** | 连接建立时间、操作响应时间、批量操作 | 1个 |
| **并发测试** | 多连接并发、多操作并发 | 2个 |
| **错误处理测试** | 无效连接、配置验证、慢命令检测 | 2个 |
| **健康检查测试** | 连接稳定性、统计信息 | 1个 |

## 🚀 运行测试

### 前置要求

1. **安装 Redis 服务器**
   ```bash
   # Windows (使用 Chocolatey)
   choco install redis-64
   
   # macOS (使用 Homebrew)
   brew install redis
   
   # Ubuntu/Debian
   sudo apt-get install redis-server
   ```

2. **启动 Redis 服务**
   ```bash
   # 启动 Redis 服务器
   redis-server
   
   # 默认运行在 localhost:6379
   ```

### 执行测试

```bash
# 运行 Redis 集成测试
cargo run --example test_redis

# 查看详细日志输出
RUST_LOG=info cargo run --example test_redis
```

### 配置自定义连接

如果您的 Redis 服务器不在默认位置，请修改 `examples/test_redis.rs` 中的配置：

```rust
// 修改连接配置
const REDIS_HOST: &str = "your-redis-host";  // 默认: "localhost"
const REDIS_PORT: u16 = 6379;               // 默认: 6379
const REDIS_DATABASE: u8 = 0;               // 默认: 0

// 如果需要密码认证，使用这个函数
fn build_redis_url_with_auth(password: &str) -> String {
    format!("redis://:{}@{}:{}/{}", password, REDIS_HOST, REDIS_PORT, REDIS_DATABASE)
}
```

## 📊 测试结果解读

### 成功执行的输出示例

```
🚀 开始 Redis 连接测试
📡 连接目标: localhost:6379
📄 数据库索引: 0
🗺️ 测试覆盖维度: 功能性、性能、并发、错误处理

🏁 正在执行: 基本连接测试
🧪 测试 1: 基本 Redis 连接
✅ 基本连接测试成功
✅ 基本连接测试 - 成功

🏁 正在执行: RedisConnection 结构体测试
🧪 测试 2: RedisConnection 结构体功能
📊 连接统计: 最大连接数=10, 最小连接数=2, 连接超时=10秒
✅ RedisConnection 结构体测试 - 成功

...

🎆 Redis 测试结果总结
✅ 成功: 13 个测试
❌ 失败: 0 个测试
🎉 所有 Redis 测试都已成功通过！
```

### 性能基准参考

- **连接建立时间**: 应该 < 100ms
- **Ping 响应时间**: 应该 < 100ms  
- **基本操作延迟**: SET/GET 应该 < 10ms
- **并发操作**: 支持多个连接同时操作

## 🔧 故障排除

### 常见问题及解决方案

1. **连接被拒绝**
   ```
   错误: Redis 连接错误: 连接失败
   ```
   **解决**: 确保 Redis 服务器已启动并运行在正确的主机和端口

2. **认证失败**
   ```
   错误: Redis 操作错误: NOAUTH Authentication required
   ```
   **解决**: 使用 `build_redis_url_with_auth()` 函数提供密码

3. **超时错误**
   ```
   错误: 操作超时: GET operation
   ```
   **解决**: 检查网络连接或增加超时配置

4. **慢命令警告**
   ```
   警告: 慢 Redis 命令: SET_BUILTIN 耗时 120ms
   ```
   **说明**: 这是正常的性能监控，可以调整 `slow_threshold_ms` 配置

### 调试建议

1. **查看详细日志**
   ```bash
   RUST_LOG=debug cargo run --example test_redis
   ```

2. **检查 Redis 服务器状态**
   ```bash
   redis-cli ping
   # 应该返回 PONG
   ```

3. **查看 Redis 服务器日志**
   ```bash
   # 查看 Redis 服务器日志以获取更多信息
   redis-cli monitor
   ```

## 🎯 扩展测试

您可以根据实际需求扩展测试：

1. **添加更多数据类型测试** (集合、有序集合等)
2. **添加持久化测试** (RDB、AOF)
3. **添加集群测试** (Redis Cluster)
4. **添加 Pub/Sub 测试** (发布订阅)
5. **添加事务测试** (MULTI/EXEC)

## 📚 相关文档

- [Redis 官方文档](https://redis.io/documentation)
- [clamber-web-core Redis 模块文档](../src/redis/README.md)
- [性能优化指南](../docs/performance-tuning.md)

---

通过这个完整的测试套件，您可以确信 Redis 模块在各种场景下都能稳定工作！