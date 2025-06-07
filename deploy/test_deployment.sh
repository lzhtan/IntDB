#!/bin/bash
#
# IntDB部署验证脚本
# 用于测试IntDB在Linux服务器上的部署状态
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
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# 检查命令是否存在
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 检查端口是否监听
check_port() {
    local port=$1
    local service=$2
    
    if netstat -tuln 2>/dev/null | grep -q ":$port "; then
        log_success "$service 正在监听端口 $port"
        return 0
    else
        log_error "$service 未在端口 $port 上监听"
        return 1
    fi
}

# 检查HTTP响应
check_http() {
    local url=$1
    local description=$2
    
    if curl -f -s "$url" >/dev/null 2>&1; then
        log_success "$description 响应正常"
        return 0
    else
        log_error "$description 无响应"
        return 1
    fi
}

# 测试API端点
test_api_endpoint() {
    local endpoint=$1
    local description=$2
    local expected_status=${3:-200}
    
    log_info "测试 $description: $endpoint"
    
    local response=$(curl -s -w "%{http_code}" "$endpoint" 2>/dev/null)
    local status_code="${response: -3}"
    local body="${response%???}"
    
    if [ "$status_code" = "$expected_status" ]; then
        log_success "$description 返回状态码 $status_code"
        if [ -n "$body" ] && [ "$body" != "000" ]; then
            echo "      响应内容: ${body:0:100}..."
        fi
        return 0
    else
        log_error "$description 返回状态码 $status_code (期望 $expected_status)"
        return 1
    fi
}

# 检查系统资源
check_system_resources() {
    log_info "检查系统资源..."
    
    # 检查CPU
    local cpu_count=$(nproc)
    log_info "CPU核心数: $cpu_count"
    
    # 检查内存
    local total_mem=$(free -h | awk '/^Mem:/ {print $2}')
    local available_mem=$(free -h | awk '/^Mem:/ {print $7}')
    log_info "总内存: $total_mem, 可用内存: $available_mem"
    
    # 检查磁盘空间
    local disk_usage=$(df -h / | awk 'NR==2 {print $5}')
    log_info "根分区使用率: $disk_usage"
    
    # 检查负载
    local load_avg=$(uptime | awk -F'load average:' '{print $2}')
    log_info "系统负载:$load_avg"
}

# 检查IntDB服务状态
check_intdb_service() {
    log_info "检查IntDB服务状态..."
    
    if systemctl is-active --quiet intdb 2>/dev/null; then
        log_success "IntDB服务运行中"
        
        # 显示服务详细状态
        echo "      $(systemctl show intdb --property=MainPID --value | xargs ps -p 2>/dev/null | tail -1 || echo 'PID信息不可用')"
        
        local start_time=$(systemctl show intdb --property=ActiveEnterTimestamp --value)
        echo "      启动时间: $start_time"
        
    elif systemctl is-enabled --quiet intdb 2>/dev/null; then
        log_warning "IntDB服务已配置但未运行"
        echo "      尝试启动: sudo systemctl start intdb"
    else
        log_error "IntDB服务未安装或未配置"
    fi
}

# 检查Docker容器(如果使用Docker部署)
check_docker_containers() {
    if command_exists docker; then
        log_info "检查Docker容器..."
        
        local intdb_container=$(docker ps --filter "name=intdb" --format "table {{.Names}}\t{{.Status}}" 2>/dev/null)
        if [ -n "$intdb_container" ]; then
            log_success "找到IntDB Docker容器"
            echo "$intdb_container"
        else
            log_warning "未找到运行中的IntDB Docker容器"
        fi
    fi
}

# 检查配置文件
check_config_files() {
    log_info "检查配置文件..."
    
    local config_files=(
        "/etc/intdb/config.toml"
        "/etc/systemd/system/intdb.service"
    )
    
    for config_file in "${config_files[@]}"; do
        if [ -f "$config_file" ]; then
            log_success "配置文件存在: $config_file"
        else
            log_warning "配置文件缺失: $config_file"
        fi
    done
}

# 检查日志文件
check_log_files() {
    log_info "检查日志文件..."
    
    local log_files=(
        "/var/log/intdb/intdb.log"
        "/opt/intdb/logs"
    )
    
    for log_path in "${log_files[@]}"; do
        if [ -e "$log_path" ]; then
            log_success "日志路径存在: $log_path"
            if [ -f "$log_path" ]; then
                local size=$(stat -c%s "$log_path" 2>/dev/null || echo "0")
                echo "      文件大小: $((size / 1024))KB"
            fi
        else
            log_warning "日志路径不存在: $log_path"
        fi
    done
    
    # 显示最近的错误日志
    if journalctl -u intdb --no-pager -n 0 >/dev/null 2>&1; then
        local error_count=$(journalctl -u intdb --no-pager --priority=err --since="1 hour ago" | wc -l)
        if [ "$error_count" -gt 0 ]; then
            log_warning "最近1小时有 $error_count 条错误日志"
            echo "      查看命令: journalctl -u intdb --priority=err --since='1 hour ago'"
        else
            log_success "最近1小时无错误日志"
        fi
    fi
}

# 性能基准测试
run_performance_test() {
    log_info "运行性能基准测试..."
    
    local base_url="http://localhost:2999"
    
    # 测试健康检查响应时间
    log_info "测试健康检查响应时间..."
    local health_time=$(curl -w "%{time_total}" -s -o /dev/null "$base_url/health" 2>/dev/null || echo "999")
    if [ "${health_time%.*}" -lt 1 ]; then
        log_success "健康检查响应时间: ${health_time}s"
    else
        log_warning "健康检查响应时间较慢: ${health_time}s"
    fi
    
    # 测试简单查询
    log_info "测试查询性能..."
    local query_data='{"path_conditions": [{"type": "through_switch", "value": {"switch_id": "test"}}]}'
    local query_time=$(curl -w "%{time_total}" -s -o /dev/null \
        -X POST "$base_url/query" \
        -H "Content-Type: application/json" \
        -d "$query_data" 2>/dev/null || echo "999")
    
    if [ "${query_time%.*}" -lt 1 ]; then
        log_success "查询响应时间: ${query_time}s"
    else
        log_warning "查询响应时间较慢: ${query_time}s"
    fi
}

# 网络连通性测试
test_network_connectivity() {
    log_info "测试网络连通性..."
    
    local host=${1:-localhost}
    local port=${2:-2999}
    
    # 测试TCP连接
    if timeout 5 bash -c "</dev/tcp/$host/$port" 2>/dev/null; then
        log_success "TCP连接到 $host:$port 成功"
    else
        log_error "TCP连接到 $host:$port 失败"
        return 1
    fi
    
    # 测试HTTP连接
    if curl -f -s --max-time 5 "http://$host:$port/health" >/dev/null 2>&1; then
        log_success "HTTP连接到 $host:$port 成功"
    else
        log_error "HTTP连接到 $host:$port 失败"
        return 1
    fi
}

# 生成测试报告
generate_report() {
    log_info "生成部署验证报告..."
    
    local report_file="/tmp/intdb_deployment_report_$(date +%Y%m%d_%H%M%S).txt"
    
    {
        echo "IntDB 部署验证报告"
        echo "=================="
        echo "生成时间: $(date)"
        echo "主机名: $(hostname)"
        echo "操作系统: $(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2 2>/dev/null || uname -a)"
        echo ""
        
        echo "系统信息:"
        echo "--------"
        echo "CPU: $(nproc) 核心"
        echo "内存: $(free -h | awk '/^Mem:/ {print $2}')"
        echo "磁盘: $(df -h / | awk 'NR==2 {print $4}') 可用"
        echo "负载: $(uptime | awk -F'load average:' '{print $2}')"
        echo ""
        
        echo "服务状态:"
        echo "--------"
        systemctl status intdb --no-pager 2>/dev/null || echo "IntDB服务未运行"
        echo ""
        
        echo "网络监听:"
        echo "--------"
        netstat -tuln | grep :2999 || echo "端口2999未监听"
        echo ""
        
        echo "最近日志 (最后10行):"
        echo "-------------------"
        journalctl -u intdb --no-pager -n 10 2>/dev/null || echo "无法获取日志"
        
    } > "$report_file"
    
    log_success "报告已生成: $report_file"
}

# 主测试函数
main() {
    echo "========================================="
    echo "      IntDB Linux 部署验证工具"
    echo "========================================="
    echo
    
    local host=${1:-localhost}
    local port=${2:-2999}
    local base_url="http://$host:$port"
    
    # 系统检查
    check_system_resources
    echo
    
    # 服务检查
    check_intdb_service
    echo
    
    # Docker检查
    check_docker_containers
    echo
    
    # 配置文件检查
    check_config_files
    echo
    
    # 日志检查
    check_log_files
    echo
    
    # 网络检查
    log_info "测试网络连接..."
    check_port "$port" "IntDB"
    test_network_connectivity "$host" "$port"
    echo
    
    # API端点测试
    log_info "测试API端点..."
    test_api_endpoint "$base_url/health" "健康检查"
    test_api_endpoint "$base_url/stats" "统计信息"
    
    # 查询测试
    log_info "测试查询API..."
    curl -X POST "$base_url/query" \
        -H "Content-Type: application/json" \
        -d '{"path_conditions": [{"type": "through_switch", "value": {"switch_id": "test"}}]}' \
        -w "状态码: %{http_code}, 响应时间: %{time_total}s\n" \
        -o /dev/null -s 2>/dev/null || log_error "查询测试失败"
    echo
    
    # 性能测试
    run_performance_test
    echo
    
    # 生成报告
    generate_report
    
    echo "========================================="
    echo "验证完成！如有问题请查看上述输出。"
    echo "========================================="
}

# 运行主函数
main "$@" 