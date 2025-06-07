#!/bin/bash

# =================================================================
# IntDB vs InfluxDB 全面性能测试套件
# =================================================================

# 配置变量
INTDB_URL="http://127.0.0.1:2999"
INFLUXDB_URL="http://127.0.0.1:8086"
RESULTS_DIR="performance_results_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# =================================================================
# 测试1: 并发数递增测试 (Concurrency Scaling Test)
# =================================================================
test_concurrency_scaling() {
    log "开始并发数递增测试..."
    
    CONCURRENCY_LEVELS=(1 5 10 20 50 100 200 500)
    TEST_DURATION=15
    
    echo "concurrency,database,transactions,availability,response_time,qps,failed_transactions,max_latency" > "$RESULTS_DIR/concurrency_scaling.csv"
    
    for concurrency in "${CONCURRENCY_LEVELS[@]}"; do
        log "测试并发数: $concurrency"
        
        # IntDB测试
        log "  测试IntDB..."
        siege -c $concurrency -t ${TEST_DURATION}s "$INTDB_URL/health" > "$RESULTS_DIR/intdb_c${concurrency}_temp.log" 2>&1
        parse_siege_results "$RESULTS_DIR/intdb_c${concurrency}_temp.log" "$concurrency" "IntDB" >> "$RESULTS_DIR/concurrency_scaling.csv"
        
        sleep 2  # 短暂休息
        
        # InfluxDB测试
        log "  测试InfluxDB..."
        siege -c $concurrency -t ${TEST_DURATION}s "$INFLUXDB_URL/ping" > "$RESULTS_DIR/influxdb_c${concurrency}_temp.log" 2>&1
        parse_siege_results "$RESULTS_DIR/influxdb_c${concurrency}_temp.log" "$concurrency" "InfluxDB" >> "$RESULTS_DIR/concurrency_scaling.csv"
        
        sleep 5  # 让系统恢复
    done
    
    success "并发数递增测试完成"
}

# =================================================================
# 测试2: 持续时间测试 (Duration Test)
# =================================================================
test_duration_scaling() {
    log "开始持续时间测试..."
    
    DURATIONS=(10 30 60 120 300)  # 秒
    CONCURRENCY=50
    
    echo "duration,database,transactions,availability,response_time,qps,failed_transactions,max_latency" > "$RESULTS_DIR/duration_scaling.csv"
    
    for duration in "${DURATIONS[@]}"; do
        log "测试持续时间: ${duration}秒"
        
        # IntDB测试
        log "  测试IntDB..."
        siege -c $CONCURRENCY -t ${duration}s "$INTDB_URL/health" > "$RESULTS_DIR/intdb_d${duration}_temp.log" 2>&1
        parse_siege_results "$RESULTS_DIR/intdb_d${duration}_temp.log" "$duration" "IntDB" >> "$RESULTS_DIR/duration_scaling.csv"
        
        sleep 2
        
        # InfluxDB测试
        log "  测试InfluxDB..."
        siege -c $CONCURRENCY -t ${duration}s "$INFLUXDB_URL/ping" > "$RESULTS_DIR/influxdb_d${duration}_temp.log" 2>&1
        parse_siege_results "$RESULTS_DIR/influxdb_d${duration}_temp.log" "$duration" "InfluxDB" >> "$RESULTS_DIR/duration_scaling.csv"
        
        sleep 10
    done
    
    success "持续时间测试完成"
}

# =================================================================
# 测试3: 功能端点对比测试 (Functional Endpoint Test)
# =================================================================
test_functional_endpoints() {
    log "开始功能端点对比测试..."
    
    CONCURRENCY=20
    DURATION=30
    
    echo "endpoint,database,transactions,availability,response_time,qps,failed_transactions,max_latency" > "$RESULTS_DIR/functional_endpoints.csv"
    
    # IntDB不同端点测试
    INTDB_ENDPOINTS=("/health" "/flows/test_flow_1")
    
    for endpoint in "${INTDB_ENDPOINTS[@]}"; do
        log "测试IntDB端点: $endpoint"
        siege -c $CONCURRENCY -t ${DURATION}s "$INTDB_URL$endpoint" > "$RESULTS_DIR/intdb_endpoint_${endpoint//\//_}_temp.log" 2>&1
        parse_siege_results "$RESULTS_DIR/intdb_endpoint_${endpoint//\//_}_temp.log" "$endpoint" "IntDB" >> "$RESULTS_DIR/functional_endpoints.csv"
        sleep 2
    done
    
    # InfluxDB不同端点测试
    INFLUXDB_ENDPOINTS=("/ping" "/health")
    
    for endpoint in "${INFLUXDB_ENDPOINTS[@]}"; do
        log "测试InfluxDB端点: $endpoint"
        siege -c $CONCURRENCY -t ${DURATION}s "$INFLUXDB_URL$endpoint" > "$RESULTS_DIR/influxdb_endpoint_${endpoint//\//_}_temp.log" 2>&1
        parse_siege_results "$RESULTS_DIR/influxdb_endpoint_${endpoint//\//_}_temp.log" "$endpoint" "InfluxDB" >> "$RESULTS_DIR/functional_endpoints.csv"
        sleep 2
    done
    
    success "功能端点测试完成"
}

# =================================================================
# 测试4: 混合负载测试 (Mixed Workload Test)
# =================================================================
test_mixed_workload() {
    log "开始混合负载测试..."
    
    # 创建URL文件
    cat > "$RESULTS_DIR/intdb_mixed_urls.txt" << EOF
$INTDB_URL/health
$INTDB_URL/flows/test_flow_1
$INTDB_URL/health
$INTDB_URL/health
EOF

    cat > "$RESULTS_DIR/influxdb_mixed_urls.txt" << EOF
$INFLUXDB_URL/ping
$INFLUXDB_URL/health
$INFLUXDB_URL/ping
$INFLUXDB_URL/ping
EOF

    echo "workload,database,transactions,availability,response_time,qps,failed_transactions,max_latency" > "$RESULTS_DIR/mixed_workload.csv"
    
    # IntDB混合负载
    log "测试IntDB混合负载..."
    siege -c 30 -t 60s -f "$RESULTS_DIR/intdb_mixed_urls.txt" > "$RESULTS_DIR/intdb_mixed_temp.log" 2>&1
    parse_siege_results "$RESULTS_DIR/intdb_mixed_temp.log" "mixed" "IntDB" >> "$RESULTS_DIR/mixed_workload.csv"
    
    sleep 5
    
    # InfluxDB混合负载
    log "测试InfluxDB混合负载..."
    siege -c 30 -t 60s -f "$RESULTS_DIR/influxdb_mixed_urls.txt" > "$RESULTS_DIR/influxdb_mixed_temp.log" 2>&1
    parse_siege_results "$RESULTS_DIR/influxdb_mixed_temp.log" "mixed" "InfluxDB" >> "$RESULTS_DIR/mixed_workload.csv"
    
    success "混合负载测试完成"
}

# =================================================================
# 测试5: 长期稳定性测试 (Long-term Stability Test)
# =================================================================
test_long_term_stability() {
    log "开始长期稳定性测试 (10分钟)..."
    
    echo "database,transactions,availability,response_time,qps,failed_transactions,max_latency" > "$RESULTS_DIR/long_term_stability.csv"
    
    # IntDB长期测试
    log "IntDB长期稳定性测试..."
    siege -c 20 -t 600s "$INTDB_URL/health" > "$RESULTS_DIR/intdb_longterm_temp.log" 2>&1 &
    INTDB_PID=$!
    
    # InfluxDB长期测试
    log "InfluxDB长期稳定性测试..."
    siege -c 20 -t 600s "$INFLUXDB_URL/ping" > "$RESULTS_DIR/influxdb_longterm_temp.log" 2>&1 &
    INFLUXDB_PID=$!
    
    # 等待完成
    wait $INTDB_PID
    wait $INFLUXDB_PID
    
    parse_siege_results "$RESULTS_DIR/intdb_longterm_temp.log" "long_term" "IntDB" >> "$RESULTS_DIR/long_term_stability.csv"
    parse_siege_results "$RESULTS_DIR/influxdb_longterm_temp.log" "long_term" "InfluxDB" >> "$RESULTS_DIR/long_term_stability.csv"
    
    success "长期稳定性测试完成"
}

# =================================================================
# 测试6: 突发负载测试 (Burst Load Test)
# =================================================================
test_burst_load() {
    log "开始突发负载测试..."
    
    echo "phase,database,transactions,availability,response_time,qps,failed_transactions,max_latency" > "$RESULTS_DIR/burst_load.csv"
    
    # 突发负载模式：低-高-低
    PHASES=("10:low" "100:burst" "10:recovery")
    
    for phase_config in "${PHASES[@]}"; do
        IFS=':' read -r concurrency phase <<< "$phase_config"
        log "突发负载阶段: $phase (并发数: $concurrency)"
        
        # IntDB
        siege -c $concurrency -t 20s "$INTDB_URL/health" > "$RESULTS_DIR/intdb_burst_${phase}_temp.log" 2>&1
        parse_siege_results "$RESULTS_DIR/intdb_burst_${phase}_temp.log" "$phase" "IntDB" >> "$RESULTS_DIR/burst_load.csv"
        
        sleep 2
        
        # InfluxDB
        siege -c $concurrency -t 20s "$INFLUXDB_URL/ping" > "$RESULTS_DIR/influxdb_burst_${phase}_temp.log" 2>&1
        parse_siege_results "$RESULTS_DIR/influxdb_burst_${phase}_temp.log" "$phase" "InfluxDB" >> "$RESULTS_DIR/burst_load.csv"
        
        sleep 5
    done
    
    success "突发负载测试完成"
}

# =================================================================
# 辅助函数: 解析siege结果
# =================================================================
parse_siege_results() {
    local log_file="$1"
    local test_param="$2"
    local database="$3"
    
    if [[ ! -f "$log_file" ]]; then
        echo "$test_param,$database,0,0,0,0,0,0"
        return
    fi
    
    local transactions=$(grep "Transactions:" "$log_file" | awk '{print $2}')
    local availability=$(grep "Availability:" "$log_file" | awk '{print $2}' | tr -d '%')
    local response_time=$(grep "Response time:" "$log_file" | awk '{print $3}')
    local transaction_rate=$(grep "Transaction rate:" "$log_file" | awk '{print $3}')
    local failed=$(grep "Failed transactions:" "$log_file" | awk '{print $3}')
    local longest=$(grep "Longest transaction:" "$log_file" | awk '{print $3}')
    
    # 处理空值
    transactions=${transactions:-0}
    availability=${availability:-0}
    response_time=${response_time:-0}
    transaction_rate=${transaction_rate:-0}
    failed=${failed:-0}
    longest=${longest:-0}
    
    echo "$test_param,$database,$transactions,$availability,$response_time,$transaction_rate,$failed,$longest"
}

# =================================================================
# 生成测试报告
# =================================================================
generate_report() {
    log "生成测试报告..."
    
    cat > "$RESULTS_DIR/test_report.md" << EOF
# IntDB vs InfluxDB 性能测试报告

## 测试时间
$(date)

## 测试环境
- 操作系统: $(uname -s) $(uname -r)
- IntDB URL: $INTDB_URL
- InfluxDB URL: $INFLUXDB_URL

## 测试结果文件
1. 并发数递增测试: \`concurrency_scaling.csv\`
2. 持续时间测试: \`duration_scaling.csv\`
3. 功能端点测试: \`functional_endpoints.csv\`
4. 混合负载测试: \`mixed_workload.csv\`
5. 长期稳定性测试: \`long_term_stability.csv\`
6. 突发负载测试: \`burst_load.csv\`

## 快速分析脚本
使用以下Python脚本分析结果:

\`\`\`python
import pandas as pd
import matplotlib.pyplot as plt

# 读取并发测试结果
df = pd.read_csv('concurrency_scaling.csv')
intdb_data = df[df['database'] == 'IntDB']
influxdb_data = df[df['database'] == 'InfluxDB']

# 绘制响应时间对比
plt.figure(figsize=(12, 8))
plt.subplot(2, 2, 1)
plt.plot(intdb_data['concurrency'], intdb_data['response_time'], 'b-o', label='IntDB')
plt.plot(influxdb_data['concurrency'], influxdb_data['response_time'], 'r-s', label='InfluxDB')
plt.xlabel('并发数')
plt.ylabel('响应时间 (ms)')
plt.title('响应时间 vs 并发数')
plt.legend()

plt.subplot(2, 2, 2)
plt.plot(intdb_data['concurrency'], intdb_data['qps'], 'b-o', label='IntDB')
plt.plot(influxdb_data['concurrency'], influxdb_data['qps'], 'r-s', label='InfluxDB')
plt.xlabel('并发数')
plt.ylabel('QPS')
plt.title('吞吐量 vs 并发数')
plt.legend()

plt.tight_layout()
plt.savefig('performance_comparison.png')
plt.show()
\`\`\`
EOF

    success "测试报告生成完成: $RESULTS_DIR/test_report.md"
}

# =================================================================
# 主函数
# =================================================================
main() {
    log "开始IntDB vs InfluxDB全面性能测试"
    log "结果将保存在: $RESULTS_DIR"
    
    # 检查服务可用性
    if ! curl -s "$INTDB_URL/health" > /dev/null; then
        error "IntDB服务不可用，请先启动IntDB"
        exit 1
    fi
    
    if ! curl -s "$INFLUXDB_URL/ping" > /dev/null; then
        error "InfluxDB服务不可用，请先启动InfluxDB"
        exit 1
    fi
    
    success "服务检查通过"
    
    # 运行所有测试
    test_concurrency_scaling
    test_duration_scaling
    test_functional_endpoints
    test_mixed_workload
    test_burst_load
    # test_long_term_stability  # 取消注释以运行长期测试
    
    # 生成报告
    generate_report
    
    success "所有测试完成！结果保存在: $RESULTS_DIR"
    log "使用以下命令查看结果："
    echo "  cd $RESULTS_DIR"
    echo "  ls -la"
    echo "  cat test_report.md"
}

# 运行主函数
main "$@" 