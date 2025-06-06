# IntDB Linux部署指南

## 🚀 快速部署

### 方式一：一键安装脚本 (推荐)

```bash
# 下载并运行安装脚本
curl -fsSL https://raw.githubusercontent.com/lzhtan/intdb/main/deploy/install.sh | sudo bash

# 启动服务
sudo systemctl start intdb
sudo systemctl enable intdb

# 测试连接
curl http://localhost:3000/health
```

### 方式二：Docker部署

```bash
# 1. 克隆仓库
git clone https://github.com/lzhtan/intdb.git
cd intdb

# 2. 构建并启动
docker-compose up -d

# 3. 测试连接
curl http://localhost:3000/health
```

### 方式三：从源码编译

```bash
# 1. 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. 克隆并编译
git clone https://github.com/lzhtan/intdb.git
cd intdb
cargo build --release --examples

# 3. 运行
./target/release/examples/api_server
```

## 📋 详细部署步骤

### 1. 系统准备

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y curl build-essential pkg-config libssl-dev git
```

**CentOS/RHEL:**
```bash
sudo yum update -y
sudo yum groupinstall -y "Development Tools"
sudo yum install -y curl openssl-devel pkg-config git
```

**Alpine Linux:**
```bash
sudo apk update
sudo apk add --no-cache curl build-base openssl-dev pkgconfig git
```

### 2. 用户和目录设置

```bash
# 创建系统用户
sudo useradd -r -s /bin/false -d /opt/intdb intdb

# 创建目录结构
sudo mkdir -p /opt/intdb/{bin,data,logs,config}
sudo mkdir -p /var/log/intdb
sudo mkdir -p /etc/intdb

# 设置权限
sudo chown -R intdb:intdb /opt/intdb
sudo chown -R intdb:intdb /var/log/intdb
sudo chown -R intdb:intdb /etc/intdb
```

### 3. 编译和安装

```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 克隆源码
git clone https://github.com/lzhtan/intdb.git
cd intdb

# 编译
cargo build --release --examples

# 安装二进制文件
sudo cp target/release/examples/api_server /opt/intdb/bin/intdb-server
sudo cp target/release/examples/test_api_server /opt/intdb/bin/intdb-test
sudo chmod +x /opt/intdb/bin/*
```

### 4. 配置文件

创建配置文件 `/etc/intdb/config.toml`:

```toml
[server]
bind = "0.0.0.0:3000"
workers = 4

[database]
data_path = "/opt/intdb/data"
max_memory_mb = 1024

[logging]
level = "info"
file = "/var/log/intdb/intdb.log"
max_size_mb = 100
max_files = 7

[performance]
query_timeout = 30
max_connections = 1000
```

### 5. Systemd服务

创建服务文件 `/etc/systemd/system/intdb.service`:

```ini
[Unit]
Description=IntDB - In-band Network Telemetry Database
Documentation=https://github.com/lzhtan/intdb
After=network.target

[Service]
Type=simple
User=intdb
Group=intdb
WorkingDirectory=/opt/intdb
ExecStart=/opt/intdb/bin/intdb-server
ExecReload=/bin/kill -HUP $MAINPID
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=intdb

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ReadWritePaths=/opt/intdb/data /var/log/intdb
ProtectHome=true

# 资源限制
LimitNOFILE=65535
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

启用服务:
```bash
sudo systemctl daemon-reload
sudo systemctl enable intdb
sudo systemctl start intdb
```

## 🔧 配置说明

### 服务器配置

- **bind**: 服务监听地址，默认`0.0.0.0:3000`
- **workers**: 工作线程数，建议设置为CPU核心数

### 数据库配置

- **data_path**: 数据存储路径
- **max_memory_mb**: 最大内存使用量(MB)

### 日志配置

- **level**: 日志级别 (error, warn, info, debug, trace)
- **file**: 日志文件路径
- **max_size_mb**: 单个日志文件最大大小
- **max_files**: 保留的日志文件数量

## 🌐 网络配置

### 防火墙设置

**Ubuntu/Debian (UFW):**
```bash
sudo ufw allow 3000/tcp comment "IntDB API"
```

**CentOS/RHEL (firewalld):**
```bash
sudo firewall-cmd --permanent --add-port=3000/tcp
sudo firewall-cmd --reload
```

**通用 (iptables):**
```bash
sudo iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
```

### Nginx反向代理

创建配置文件 `/etc/nginx/sites-available/intdb`:

```nginx
server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

启用站点:
```bash
sudo ln -s /etc/nginx/sites-available/intdb /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## 📊 监控和日志

### 服务状态查看

```bash
# 查看服务状态
sudo systemctl status intdb

# 查看实时日志
sudo journalctl -u intdb -f

# 查看错误日志
sudo journalctl -u intdb --priority=err
```

### 健康检查

```bash
# API健康检查
curl http://localhost:3000/health

# 性能测试
curl -X POST http://localhost:3000/query \
  -H 'Content-Type: application/json' \
  -d '{"path_conditions": [{"type": "through_switch", "value": {"switch_id": "s1"}}]}'
```

### 日志文件位置

- 系统日志: `journalctl -u intdb`
- 应用日志: `/var/log/intdb/intdb.log`
- 错误日志: `/var/log/intdb/error.log`

## 🚀 性能优化

### 系统优化

```bash
# 增加文件描述符限制
echo "intdb soft nofile 65535" >> /etc/security/limits.conf
echo "intdb hard nofile 65535" >> /etc/security/limits.conf

# 启用TCP BBR拥塞控制
echo "net.core.default_qdisc=fq" >> /etc/sysctl.conf
echo "net.ipv4.tcp_congestion_control=bbr" >> /etc/sysctl.conf
sysctl -p
```

### 应用优化

- 调整工作线程数匹配CPU核心数
- 根据内存大小调整`max_memory_mb`
- 启用日志轮转避免磁盘空间不足

## 🔒 安全建议

1. **防火墙配置**: 只开放必要端口
2. **SSL/TLS**: 使用Nginx提供HTTPS支持
3. **用户权限**: 使用专用系统用户运行服务
4. **定期更新**: 保持系统和依赖更新
5. **监控日志**: 监控异常访问和错误

## 🆘 故障排除

### 常见问题

**服务启动失败:**
```bash
# 检查配置文件语法
sudo -u intdb /opt/intdb/bin/intdb-server --check-config

# 检查文件权限
ls -la /opt/intdb/
```

**端口被占用:**
```bash
# 查看端口占用
sudo netstat -tulpn | grep :3000
sudo lsof -i :3000
```

**内存不足:**
```bash
# 检查内存使用
free -h
# 调整配置文件中的max_memory_mb参数
```

**日志文件过大:**
```bash
# 手动轮转日志
sudo logrotate -f /etc/logrotate.d/intdb
```

### 联系支持

- GitHub Issues: https://github.com/lzhtan/intdb/issues
- 文档: https://github.com/lzhtan/intdb/docs
- Email: support@intdb.org 