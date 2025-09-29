# 静态文件目录

这个目录包含代理服务器提供的静态文件。

## 文件说明

- `index.html` - 代理服务器的主页面，包含 API 文档和测试界面
- `README.md` - 本文件

## 访问方式

- 根路径: `http://localhost:8080/`
- 静态文件: `http://localhost:8080/static/`

## API 路由

### Kafka API (转发到端口 3000)
- `/api/kafka/` - Kafka API 根路径
- `/api/kafka/health` - 健康检查
- `/api/kafka/send-message` - 发送消息
- `/api/kafka/send-user-message` - 发送用户消息
- `/api/kafka/producer-stats` - 生产者统计
- `/api/kafka/consumer-stats` - 消费者统计

### 配置 API (转发到端口 3001)
- `/api/config/` - 配置 API 根路径
- `/api/config/health` - 健康检查
- `/api/config/send-message` - 发送消息
- `/api/config/producer-stats` - 生产者统计
- `/api/config/consumer-stats` - 消费者统计
