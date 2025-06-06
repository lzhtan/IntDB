#!/bin/bash
#
# IntDB Linux 安装脚本
# 支持 Ubuntu/Debian/CentOS/RHEL/Amazon Linux
#

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检测操作系统
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
        VER=$VERSION_ID
    else
        log_error "无法检测操作系统版本"
        exit 1
    fi
    
    log_info "检测到操作系统: $PRETTY_NAME"
}

# 安装依赖
install_dependencies() {
    log_info "安装系统依赖..."
    
    case $OS in
        ubuntu|debian)
            apt-get update
            apt-get install -y curl build-essential pkg-config libssl-dev
            ;;
        centos|rhel|amzn)
            yum update -y
            yum groupinstall -y "Development Tools"
            yum install -y curl openssl-devel pkg-config
            ;;
        alpine)
            apk update
            apk add --no-cache curl build-base openssl-dev pkgconfig
            ;;
        *)
            log_warning "未知的操作系统: $OS，尝试通用安装..."
            ;;
    esac
}

# 安装Rust
install_rust() {
    if command -v rustc >/dev/null 2>&1; then
        local rust_version=$(rustc --version | cut -d' ' -f2)
        log_info "Rust已安装: $rust_version"
        
        # 检查版本是否满足要求
        if [[ "$rust_version" < "1.70" ]]; then
            log_warning "Rust版本过低，需要1.70+，正在更新..."
            rustup update stable
        fi
    else
        log_info "安装Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    fi
}

# 创建系统用户
create_user() {
    if ! id "intdb" >/dev/null 2>&1; then
        log_info "创建intdb系统用户..."
        useradd -r -s /bin/false -d /opt/intdb intdb
    fi
}

# 创建目录结构
create_directories() {
    log_info "创建目录结构..."
    mkdir -p /opt/intdb/{bin,data,logs,config}
    mkdir -p /var/log/intdb
    mkdir -p /etc/intdb
    
    chown -R intdb:intdb /opt/intdb
    chown -R intdb:intdb /var/log/intdb
    chown -R intdb:intdb /etc/intdb
}

# 编译IntDB
build_intdb() {
    log_info "编译IntDB..."
    
    if [ ! -d "IntDB" ]; then
        log_info "克隆IntDB源码..."
        git clone https://github.com/lzhtan/intdb.git IntDB
    fi
    
    cd IntDB
    
    # 编译release版本
    cargo build --release
    
    # 复制二进制文件
    cp target/release/intdb /opt/intdb/bin/ 2>/dev/null || {
        # 如果没有主二进制文件，复制示例
        cargo build --release --examples
        cp target/release/examples/api_server /opt/intdb/bin/intdb-server
        cp target/release/examples/test_api_server /opt/intdb/bin/intdb-test
    }
    
    chmod +x /opt/intdb/bin/*
    
    cd ..
}

# 创建配置文件
create_config() {
    log_info "创建配置文件..."
    
    cat > /etc/intdb/config.toml << 'EOF'
# IntDB 配置文件

[server]
# 服务器监听地址
bind = "0.0.0.0:3000"
# 工作线程数
workers = 4

[database]
# 数据存储路径
data_path = "/opt/intdb/data"
# 最大内存使用量(MB)
max_memory_mb = 1024

[logging]
# 日志级别: error, warn, info, debug, trace
level = "info"
# 日志文件路径
file = "/var/log/intdb/intdb.log"
# 日志轮转大小(MB)
max_size_mb = 100
# 保留的日志文件数量
max_files = 7

[performance]
# 查询超时时间(秒)
query_timeout = 30
# 最大并发连接数
max_connections = 1000
EOF

    chown intdb:intdb /etc/intdb/config.toml
}

# 创建systemd服务
create_systemd_service() {
    log_info "创建systemd服务..."
    
    cat > /etc/systemd/system/intdb.service << 'EOF'
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
EOF

    systemctl daemon-reload
}

# 设置防火墙规则
setup_firewall() {
    log_info "配置防火墙..."
    
    if command -v ufw >/dev/null 2>&1; then
        # Ubuntu/Debian UFW
        ufw allow 3000/tcp comment "IntDB API"
    elif command -v firewall-cmd >/dev/null 2>&1; then
        # CentOS/RHEL firewalld
        firewall-cmd --permanent --add-port=3000/tcp
        firewall-cmd --reload
    elif command -v iptables >/dev/null 2>&1; then
        # 通用iptables
        iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
        # 保存规则(根据发行版可能不同)
        if [ -f /etc/debian_version ]; then
            iptables-save > /etc/iptables/rules.v4
        elif [ -f /etc/redhat-release ]; then
            service iptables save
        fi
    fi
}

# 主安装函数
main() {
    log_info "开始安装IntDB..."
    
    # 检查root权限
    if [ "$EUID" -ne 0 ]; then
        log_error "请使用root权限运行此脚本"
        exit 1
    fi
    
    detect_os
    install_dependencies
    install_rust
    create_user
    create_directories
    build_intdb
    create_config
    create_systemd_service
    setup_firewall
    
    log_success "IntDB安装完成！"
    echo
    echo "📋 管理命令:"
    echo "  启动服务: systemctl start intdb"
    echo "  停止服务: systemctl stop intdb"
    echo "  查看状态: systemctl status intdb"
    echo "  开机启动: systemctl enable intdb"
    echo "  查看日志: journalctl -u intdb -f"
    echo
    echo "🌐 访问地址: http://$(hostname -I | awk '{print $1}'):3000"
    echo "🧪 测试命令: curl http://localhost:3000/health"
    echo
    echo "📁 重要路径:"
    echo "  配置文件: /etc/intdb/config.toml"
    echo "  数据目录: /opt/intdb/data"
    echo "  日志文件: /var/log/intdb/intdb.log"
    echo
}

# 运行主函数
main "$@" 