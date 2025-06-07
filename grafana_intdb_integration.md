# Grafana + IntDB 集成指南

## 🎯 概述

IntDB现在完全支持与Grafana集成，提供强大的网络流量可视化能力。本指南将帮你完成完整的集成配置。

## 🔧 集成方式

### 方式1: Prometheus数据源（推荐）

IntDB提供标准的Prometheus格式指标，Grafana可以直接采集。

#### 配置步骤：

1. **在Grafana中添加Prometheus数据源**
   - URL: `http://localhost:2999/metrics`
   - Access: Server (Default)
   - Scrape interval: 15s

2. **可用指标：**
   ```prometheus
   intdb_flows_total                    # 总流数量
   intdb_uptime_seconds                 # 服务运行时间
   intdb_memory_usage_estimate_bytes    # 内存使用估计
   intdb_api_health                     # 服务健康状态
   ```

### 方式2: JSON API数据源

使用Grafana的JSON数据源插件直接查询IntDB API。

#### 配置步骤：

1. **安装JSON数据源插件**
   ```bash
   grafana-cli plugins install marcusolsson-json-datasource
   ```

2. **添加JSON数据源**
   - URL: `http://localhost:2999/grafana/query`
   - Access: Server (Default)

3. **支持的查询指标：**
   - `flow_count` - 网络流数量
   - `avg_delay` - 平均延迟
   - `avg_queue_util` - 平均队列利用率

## 📊 仪表板示例

### 基础网络监控仪表板

```json
{
  "dashboard": {
    "title": "IntDB 网络流量监控",
    "panels": [
      {
        "title": "网络流数量",
        "type": "stat",
        "targets": [
          {
            "expr": "intdb_flows_total",
            "legendFormat": "总流数"
          }
        ]
      },
      {
        "title": "平均延迟趋势",
        "type": "timeseries",
        "targets": [
          {
            "target": "avg_delay",
            "datasource": "IntDB-JSON"
          }
        ]
      },
      {
        "title": "队列利用率",
        "type": "gauge",
        "targets": [
          {
            "target": "avg_queue_util", 
            "datasource": "IntDB-JSON"
          }
        ]
      },
      {
        "title": "服务健康状态",
        "type": "stat",
        "targets": [
          {
            "expr": "intdb_api_health",
            "legendFormat": "健康状态"
          }
        ]
      }
    ]
  }
}
```

## 🚀 快速启动

### 1. 启动IntDB
```bash
# 确保IntDB运行在2999端口
cargo run
```

### 2. 启动Grafana
```bash
# 使用Docker启动Grafana
docker run -d \
  --name grafana-intdb \
  -p 3000:3000 \
  grafana/grafana-enterprise

# 或者如果你有现有的Grafana
# 确保它运行在3000端口（不与IntDB冲突）
```

### 3. 验证连接

访问 `http://localhost:3000` 进入Grafana，然后：

1. **测试Prometheus数据源**
   ```bash
   curl http://localhost:2999/metrics
   ```

2. **测试JSON API**
   ```bash
   curl -X POST http://localhost:2999/grafana/query \
     -H "Content-Type: application/json" \
     -d '{
       "range": {"from": "2024-12-01T00:00:00Z", "to": "2024-12-31T23:59:59Z"},
       "targets": [{"target": "flow_count"}]
     }'
   ```

## 📈 高级用法

### 自定义查询

IntDB支持复杂的网络查询，可以通过Grafana变量实现：

```json
{
  "targets": [
    {
      "target": "flows_through_switch",
      "switch_id": "$switch_variable"
    }
  ]
}
```

### 告警配置

基于IntDB指标设置网络告警：

```yaml
# 延迟告警
alert:
  name: "高延迟告警"
  condition: "avg_delay > 500"
  message: "网络延迟超过500ns"

# 队列利用率告警  
alert:
  name: "队列拥塞告警"
  condition: "avg_queue_util > 0.8"
  message: "网络队列利用率超过80%"
```

## 📊 创建图表和仪表板

### 步骤1: 添加数据源

1. **登录Grafana**
   - 访问 `http://localhost:3000`
   - 用户名: `admin`，密码: `admin`

2. **配置Prometheus数据源**
   - 进入 `Configuration` → `Data Sources`
   - 点击 `Add data source`
   - 选择 `Prometheus`
   - 配置设置:
     ```
     Name: IntDB-Prometheus
     URL: http://localhost:2999
     Access: Server (default)
     HTTP Method: GET
     ```
   - 点击 `Save & Test`

### 步骤2: 创建仪表板

1. **新建仪表板**
   - 点击 `+` → `Dashboard`
   - 点击 `Add new panel`

2. **图表1: 网络流数量**
   ```
   查询: intdb_flows_total
   图表类型: Stat
   标题: 网络流总数
   单位: short
   ```

3. **图表2: 服务运行时间**
   ```
   查询: intdb_uptime_seconds
   图表类型: Stat  
   标题: 服务运行时间
   单位: seconds
   ```

4. **图表3: 内存使用情况**
   ```
   查询: intdb_memory_usage_estimate_bytes
   图表类型: Gauge
   标题: 内存使用估计
   单位: bytes
   最小值: 0
   最大值: 1073741824 (1GB)
   ```

5. **图表4: 服务健康状态**
   ```
   查询: intdb_api_health
   图表类型: Stat
   标题: 服务状态
   映射: 0=离线, 1=在线
   颜色: 红色(0), 绿色(1)
   ```

### 步骤3: 设置刷新间隔和时间范围

1. **时间范围设置**
   - 点击右上角时间选择器
   - 推荐设置: `Last 5 minutes` 或 `Last 15 minutes`
   - 也可以设置相对时间: `now-5m to now`

2. **自动刷新设置**
   - 在时间选择器旁边点击刷新图标
   - 选择自动刷新间隔:
     ```
     推荐设置:
     - 开发环境: 5s 或 10s
     - 生产环境: 30s 或 1m
     ```

3. **仪表板级别设置**
   - 点击仪表板设置齿轮图标
   - `General` → `Auto-refresh`: 设置默认刷新间隔
   - `Time options` → `Timezone`: 设置时区
   - `Time options` → `Refresh intervals`: 自定义可选的刷新间隔

### 步骤4: 高级图表配置

1. **时间序列图表 - 流量趋势**
   ```json
   {
     "查询": "intdb_flows_total",
     "图表类型": "Time series",
     "标题": "流量变化趋势",
     "Y轴": {
       "单位": "short",
       "最小值": 0
     },
     "图例": {
       "显示": true,
       "位置": "bottom"
     }
   }
   ```

2. **表格视图 - 指标汇总**
   ```json
   {
     "查询": [
       "intdb_flows_total",
       "intdb_uptime_seconds", 
       "intdb_api_health"
     ],
     "图表类型": "Table",
     "标题": "系统指标汇总",
     "列设置": {
       "时间戳": "隐藏",
       "指标名": "显示",
       "当前值": "显示"
     }
   }
   ```

### 步骤5: 告警配置

1. **创建告警规则**
   - 在图表编辑页面，点击 `Alert` 标签
   - 点击 `Create Alert`

2. **服务健康告警**
   ```
   条件: intdb_api_health < 1
   评估间隔: 10s
   持续时间: 30s
   消息: IntDB服务离线
   ```

3. **流量异常告警**
   ```
   条件: intdb_flows_total > 1000
   评估间隔: 1m
   持续时间: 2m
   消息: 网络流量异常增高
   ```

### 步骤6: 保存和共享

1. **保存仪表板**
   - 点击右上角保存图标
   - 输入名称: `IntDB 网络监控`
   - 添加标签: `network`, `intdb`, `monitoring`

2. **导出配置**
   - 仪表板设置 → `JSON Model`
   - 复制JSON配置用于备份或分享

3. **使用预定义配置**
   - 我们提供了完整的仪表盘配置文件: `Grafana/intdb_dashboard.json`
   - 导入方式：
     1. 进入Grafana → `+` → `Import`
     2. 点击 `Upload JSON file` 选择 `intdb_dashboard.json`
     3. 或者直接复制JSON内容粘贴到文本框
     4. 点击 `Load` → `Import`
   - 该配置包含4个预配置面板：
     - 启动时间 (Stat)
     - 健康度 (Time series)
     - 活跃流数量 (Stat)  
     - 内存占用 (Time series)

## 🔍 故障排除

### 常见问题

1. **连接失败**
   - 检查IntDB是否运行在2999端口
   - 确认防火墙设置
   - 验证URL: `http://localhost:2999` (不要包含/metrics)

2. **数据源错误**
   - 验证URL格式正确
   - 检查Grafana插件是否安装
   - 测试数据源连接

3. **指标为空**
   - 确认IntDB中有数据
   - 检查时间范围设置
   - 验证查询语法

4. **图表不更新**
   - 检查自动刷新设置
   - 确认时间范围覆盖数据时间
   - 验证数据源状态

### 调试命令

```bash
# 检查IntDB健康状态
curl http://localhost:2999/health

# 查看Prometheus指标
curl http://localhost:2999/metrics

# 测试Prometheus API
curl "http://localhost:2999/api/v1/query?query=intdb_flows_total"

# 测试Grafana查询
curl -X POST http://localhost:2999/grafana/query \
  -H "Content-Type: application/json" \
  -d '{"range": {"from": "now-1h", "to": "now"}, "targets": [{"target": "flow_count"}]}'
```

