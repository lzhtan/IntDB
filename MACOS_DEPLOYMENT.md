# IntDB macOS 部署指南

IntDB完全支持在macOS上运行，本指南详细说明如何在macOS环境中部署和使用IntDB。

## 系统要求

### 最低要求
- **操作系统**: macOS 10.15 (Catalina) 或更高版本
- **内存**: 512MB RAM（推荐2GB+）
- **磁盘空间**: 100MB（编译需要额外1GB临时空间）
- **网络**: 开放端口2999（可配置）

### 推荐配置
- **操作系统**: macOS 12+ (Monterey)
- **内存**: 4GB+ RAM
- **CPU**: Apple Silicon (M1/M2) 或 Intel x64
- **磁盘空间**: 500MB+

## 部署方法

### 方法1：直接编译运行（推荐）

#### 1. 安装Rust开发环境
```bash
# 安装Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

#### 2. 获取源码
```bash
# 克隆项目
git clone https://github.com/lzhtan/IntDB.git
cd IntDB

# 查看项目结构
ls -la
```

#### 3. 编译项目
```bash
# Debug版本（开发使用）
cargo build

# Release版本（生产使用）
cargo build --release
```

#### 4. 启动服务
```bash
# 启动测试服务器（包含示例数据）
cargo run --example test_api_server

# 或启动基础API服务器
cargo run --example api_server
```

#### 5. 验证部署
```bash
# 新开终端窗口，测试API
curl http://127.0.0.1:2999/health

# 应该返回类似：
# {"status":"healthy","version":"0.1.0","uptime_seconds":5,"flow_count":3}
```

### 方法2：使用Homebrew（未来支持）

```bash
# 计划支持的安装方式
brew tap lzhtan/intdb
brew install intdb

# 启动服务
intdb start --port 2999
```

### 方法3：Docker运行

```bash
# 使用Docker（需要先安装Docker Desktop for Mac）
docker build -t intdb:latest .
docker run -p 2999:2999 intdb:latest
```

## macOS特定配置

### 1. 网络配置
```bash
# 检查端口占用
lsof -i :2999

# 如果端口被占用，可以更改端口
cargo run --example api_server -- --port 3001
```

### 2. 防火墙设置
```bash
# macOS防火墙通常不会阻止本地服务
# 如果需要外部访问，在系统偏好设置 > 安全性与隐私 > 防火墙中添加例外
```

### 3. 内存优化
```bash
# 查看系统内存
system_profiler SPHardwareDataType | grep Memory

# 如果内存较少，可以限制IntDB内存使用
export INTDB_MAX_MEMORY=512  # 限制为512MB
cargo run --example api_server
```

## API使用示例

### 基础健康检查
```bash
curl http://127.0.0.1:2999/health
```

### 查询流数据
```bash
# 获取特定流
curl http://127.0.0.1:2999/flows/test_flow_1

# 获取统计信息
curl http://127.0.0.1:2999/stats
```

### 高级查询
```bash
# 路径查询
curl -X POST http://127.0.0.1:2999/query \
  -H 'Content-Type: application/json' \
  -d '{"path_conditions": [{"contains": ["s1", "s2"]}]}'

# 时间范围查询
curl -X POST http://127.0.0.1:2999/query \
  -H 'Content-Type: application/json' \
  -d '{"time_conditions": [{"after": "2025-01-01T00:00:00Z"}]}'
```

### 数据写入
```bash
curl -X POST http://127.0.0.1:2999/flows \
  -H 'Content-Type: application/json' \
  -d '{
    "flow": {
      "path": ["s1", "s2", "s4"],
      "hops": [
        {
          "hop_index": 0,
          "switch_id": "s1",
          "timestamp": "2025-06-06T10:00:00Z",
          "metrics": {
            "queue_util": 0.75,
            "delay_ns": 180,
            "bandwidth_bps": 1100
          }
        }
      ]
    }
  }'
```

## 开发配置

### 启用日志
```bash
# 设置日志级别
export RUST_LOG=debug
cargo run --example test_api_server

# 或只显示IntDB日志
export RUST_LOG=intdb=debug,info
```

### 性能监控
```bash
# 安装htop监控系统资源
brew install htop
htop

# 监控网络连接
netstat -an | grep :2999
```

### 开发工具
```bash
# 安装有用的开发工具
brew install jq  # JSON格式化
brew install httpie  # 更友好的HTTP客户端

# 使用示例
curl -s http://127.0.0.1:2999/health | jq .
http GET http://127.0.0.1:2999/flows/test_flow_1
```

## 故障排除

### 编译问题
```bash
# 清理缓存重新编译
cargo clean
cargo build

# 更新依赖
cargo update
```

### 运行时问题
```bash
# 检查端口占用
lsof -i :2999
kill -9 <PID>  # 如果需要强制关闭

# 检查系统资源
top -n 1 | grep intdb
```

### 常见错误

**1. 端口已被占用**
```bash
Error: Address already in use (os error 48)
```
解决：更换端口或关闭占用进程

**2. 内存不足**
```bash
Error: Cannot allocate memory
```
解决：关闭其他应用或增加虚拟内存

**3. 网络连接被拒绝**
```bash
curl: (7) Failed to connect
```
解决：确认服务已启动且端口正确

## 性能测试

### 压力测试
```bash
# 安装压测工具
brew install siege

# 简单压测
siege -c 10 -t 30s http://127.0.0.1:2999/health

# 复杂场景压测
siege -c 5 -t 60s -f urls.txt
```

### 内存监控
```bash
# 监控IntDB内存使用
ps -o pid,ppid,pmem,pcpu,comm -p $(pgrep test_api_server)

# 详细内存分析
sudo leaks $(pgrep test_api_server)
```

## 生产部署建议

### 1. 服务管理
```bash
# 使用launchd创建系统服务
# 创建 ~/Library/LaunchAgents/com.intdb.server.plist
```

### 2. 日志管理
```bash
# 设置日志轮转
export RUST_LOG=info
cargo run --example api_server 2>&1 | tee /var/log/intdb.log
```

### 3. 监控集成
```bash
# 集成Prometheus监控（如果需要）
# IntDB会在未来版本提供metrics端点
```

## 与其他工具集成

### 1. Grafana可视化
```bash
# 安装Grafana
brew install grafana
brew services start grafana

# 访问 http://localhost:2999（注意端口冲突）
```

### 2. Nginx反向代理
```bash
# 安装Nginx
brew install nginx

# 配置代理到IntDB
# /usr/local/etc/nginx/nginx.conf
```

## 卸载

```bash
# 删除项目目录
rm -rf IntDB/

# 如果使用了系统服务
launchctl unload ~/Library/LaunchAgents/com.intdb.server.plist
rm ~/Library/LaunchAgents/com.intdb.server.plist
```

---

**IntDB在macOS上运行完美，支持所有核心功能！**

需要帮助？请提交 [GitHub Issue](https://github.com/lzhtan/IntDB/issues) 