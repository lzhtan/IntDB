# IntDB 实际部署指南

## 🚀 Linux服务器部署方案

### 前提条件
- Linux服务器（Ubuntu 18.04+, CentOS 7+, Debian 9+等）
- 管理员权限（sudo）
- 网络连接正常

## 方案一：手动编译部署（推荐用于生产环境）

### 1. 系统准备

**Ubuntu/Debian系统：**
```bash
sudo apt update
sudo apt install -y curl build-essential pkg-config libssl-dev git
```

**CentOS/RHEL系统：**
```bash
sudo yum update -y
sudo yum groupinstall -y "Development Tools"
sudo yum install -y curl openssl-devel pkg-config git
```

### 2. 安装Rust编译环境

```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 3. 获取IntDB源码

**如果您有GitHub仓库：**
```bash
git clone https://github.com/YOUR_USERNAME/IntDB.git
cd IntDB
```

**如果没有仓库，本地传输：**
```bash
# 在本地打包（排除大文件）
tar --exclude='target' --exclude='.git' -czf intdb-source.tar.gz .

# 上传到服务器
scp intdb-source.tar.gz user@your-server:/tmp/

# 在服务器解压
ssh user@your-server
cd /tmp && tar -xzf intdb-source.tar.gz
sudo mv IntDB /opt/intdb
cd /opt/intdb
```

### 4. 编译IntDB

```bash
# 编译release版本（生产优化）
cargo build --release --examples

# 验证编译结果
ls -la target/release/examples/
```

### 5. 系统服务配置

```bash
# 创建系统用户
sudo useradd -r -s /bin/false -d /opt/intdb intdb

# 设置权限
sudo chown -R intdb:intdb /opt/intdb
sudo mkdir -p /var/log/intdb
sudo chown intdb:intdb /var/log/intdb

# 复制二进制文件到系统路径
sudo cp target/release/examples/api_server /usr/local/bin/intdb-server
sudo chmod +x /usr/local/bin/intdb-server
```

### 6. 创建systemd服务

```bash
sudo tee /etc/systemd/system/intdb.service > /dev/null <<'EOF'
[Unit]
Description=IntDB - In-band Network Telemetry Database
After=network.target

[Service]
Type=simple
User=intdb
Group=intdb
WorkingDirectory=/opt/intdb
ExecStart=/usr/local/bin/intdb-server
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# 安全设置
NoNewPrivileges=true
PrivateTmp=true
ProtectHome=true
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF

# 启用并启动服务
sudo systemctl daemon-reload
sudo systemctl enable intdb
sudo systemctl start intdb
```

### 7. 验证部署

```bash
# 检查服务状态
sudo systemctl status intdb

# 测试API
curl http://localhost:3000/health

# 查看日志
sudo journalctl -u intdb -f
```

## 方案二：Docker容器部署（推荐用于快速测试）

### 1. 安装Docker

**Ubuntu/Debian：**
```bash
sudo apt update
sudo apt install -y docker.io docker-compose
sudo systemctl start docker
sudo systemctl enable docker
```

**CentOS/RHEL：**
```bash
sudo yum install -y docker docker-compose
sudo systemctl start docker
sudo systemctl enable docker
```

### 2. 准备项目文件

```bash
# 传输项目到服务器
scp -r . user@your-server:/opt/intdb/
ssh user@your-server
cd /opt/intdb
```

### 3. 构建和运行

```bash
# 构建镜像
sudo docker build -t intdb:latest .

# 直接运行容器
sudo docker run -d \
  --name intdb-server \
  -p 3000:3000 \
  --restart unless-stopped \
  intdb:latest

# 或使用docker-compose
sudo docker-compose up -d
```

### 4. 验证Docker部署

```bash
# 检查容器状态
sudo docker ps

# 测试API
curl http://localhost:3000/health

# 查看容器日志
sudo docker logs intdb-server -f
```

## 方案三：最小化手动部署

如果编译环境有问题，可以尝试最小化部署：

### 1. 在本地编译

```bash
# 在您的开发机器上编译
cargo build --release --examples

# 打包二进制文件
tar -czf intdb-binaries.tar.gz \
  target/release/examples/api_server \
  target/release/examples/test_api_server
```

### 2. 传输到服务器

```bash
# 上传二进制文件
scp intdb-binaries.tar.gz user@your-server:/tmp/

# 在服务器上安装
ssh user@your-server
cd /tmp && tar -xzf intdb-binaries.tar.gz
sudo mv target/release/examples/api_server /usr/local/bin/intdb-server
sudo chmod +x /usr/local/bin/intdb-server

# 直接运行（临时测试）
/usr/local/bin/intdb-server
```

## 🔧 配置和优化

### 防火墙配置

```bash
# Ubuntu/Debian (UFW)
sudo ufw allow 3000/tcp

# CentOS/RHEL (firewalld)
sudo firewall-cmd --permanent --add-port=3000/tcp
sudo firewall-cmd --reload

# 通用iptables
sudo iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
```

### 反向代理（可选）

**Nginx配置：**
```bash
sudo tee /etc/nginx/sites-available/intdb > /dev/null <<'EOF'
server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
EOF

sudo ln -s /etc/nginx/sites-available/intdb /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## 🆘 故障排除

### 常见问题

1. **编译失败：**
   ```bash
   # 检查Rust版本
   rustc --version
   # 更新Rust
   rustup update stable
   ```

2. **端口被占用：**
   ```bash
   sudo netstat -tulpn | grep :3000
   sudo lsof -i :3000
   ```

3. **权限问题：**
   ```bash
   sudo chown -R intdb:intdb /opt/intdb
   sudo chmod +x /usr/local/bin/intdb-server
   ```

4. **服务启动失败：**
   ```bash
   sudo journalctl -u intdb --no-pager
   sudo systemctl status intdb
   ```

## 📊 验证部署成功

运行以下命令验证部署：

```bash
# 健康检查
curl http://localhost:3000/health
# 预期输出: {"status":"healthy","version":"0.1.0",...}

# 测试查询
curl -X POST http://localhost:3000/query \
  -H 'Content-Type: application/json' \
  -d '{"path_conditions": []}'
# 预期输出: {"flow_ids":[],"flows":null,...}

# 检查服务状态
sudo systemctl status intdb
# 预期: Active (running)
```

如果所有测试都通过，说明IntDB已成功部署在Linux服务器上！🎉 