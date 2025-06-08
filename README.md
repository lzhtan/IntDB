# IntDB
IntDB是一个面向带内网络遥测的时空数据库。

> 🎬 **项目演示视频**：[IntDB介绍视频](https://www.bilibili.com/video/BV1PpT2zzELD/)

> 🚀 **快速部署**：[Linux服务器部署](./LINUX_DEPLOYMENT.md) | [macOS本地开发](./MACOS_DEPLOYMENT.md)

## 📖 文档导航

| 内容 | 链接 | 说明 |
|------|------|------|
| **🛠️ Linux部署** | [LINUX_DEPLOYMENT.md](./LINUX_DEPLOYMENT.md) | Ubuntu/CentOS环境部署 |
| **🍎 macOS部署** | [MACOS_DEPLOYMENT.md](./MACOS_DEPLOYMENT.md) | macOS环境搭建 |
| **🐳 Docker部署** | [docker-compose.yml](./docker-compose.yml) | 容器化一键部署 |
| **📊 性能测试** | [PerformanceTestResult.md](./PerformanceTestResult.md) | 与InfluxDB的性能对比测试 |
| **📈 Grafana集成** | [grafana_intdb_integration.md](./grafana_intdb_integration.md) | 可视化监控仪表盘集成 |
| **💻 API示例** | [examples/](./examples/) | 代码示例和演示 |

## 设计理念与定位

IntDB**不是**传统时序数据库的替代品，而是专门为**带内网络遥测**场景设计的时空数据库。

### 与InfluxDB的关系：互补而非竞争

| 场景类型 | InfluxDB | IntDB | 推荐选择 |
|----------|----------|-------|----------|
| **传统时序监控** | ✅ 原生优化 | ⚠️ 基本支持 | InfluxDB |
| **简单指标聚合** | ✅ 高度优化 | ⚠️ 性能一般 | InfluxDB |
| **路径级网络分析** | ❌ 需要复杂JOIN | ✅ 原生支持 | **IntDB** |
| **跳间关联查询** | ❌ 几乎无法实现 | ✅ 高效支持 | **IntDB** |
| **流路径重构** | ❌ 性能极差 | ✅ 快速响应 | **IntDB** |
| **网络瓶颈检测** | ❌ 需要应用层处理 | ✅ 数据库级支持 | **IntDB** |

## 数据格式示例

### INT数据输入格式
```json
{
  "flow_id": "17343111536",
  "telemetry": [
    {
      "switch_id": "s1",
      "timestamp": "2025-04-21T10:00:00Z",
      "queue_util": 0.72,
      "delay_ns": 600
    },
    {
      "switch_id": "s2", 
      "timestamp": "2025-04-21T10:00:01Z",
      "queue_util": 0.64,
      "delay_ns": 580
    },
    {
      "switch_id": "s3",
      "timestamp": "2025-04-21T10:00:02Z", 
      "queue_util": 0.01,
      "delay_ns": 510
    }
  ]
}
```

### 内部存储格式

#### 流记录结构 (时空数据库格式)
```
Spatiotemporal Flow Record:
├── flow_id: "17343111536"
├── spatial_metadata: {
    ├── path_signature: "sha256(s1->s2->s3)"  // 必需：逻辑路径标识
    ├── logical_path: ["s1", "s2", "s3"]      // 必需：逻辑交换机序列
    ├── topology_coordinates: [               // 可选：物理拓扑坐标
        ├── {switch: "s1", topo_x: 100, topo_y: 200, zone: "rack1"}
        ├── {switch: "s2", topo_x: 300, topo_y: 200, zone: "rack2"}  
        └── {switch: "s3", topo_x: 500, topo_y: 200, zone: "rack3"}
    ] | null
    ├── path_geometry: "LINESTRING(100 200, 300 200, 500 200)" | null  // 可选：GIS几何
    ├── spatial_extent: {min_x: 100, min_y: 200, max_x: 500, max_y: 200} | null  // 可选
    ├── adjacency_matrix: [[0,1,0], [1,0,1], [0,1,0]] | null  // 可选：邻接关系
    └── has_spatial_info: true | false        // 标识：是否包含空间信息
}
├── temporal_metadata: {
    ├── flow_state: "active" | "completed" | "timeout"  // 必需
    ├── creation_time: 1640995200000          // 必需
    ├── last_update: 1640995800000            // 必需
    ├── window_duration: 60000                // 可选：默认60秒
    └── retention_policy: "7d"                // 可选：默认7天
}
├── spatiotemporal_windows: [
    ├── {
        ├── st_window_id: "st_w1_1640995200_logical" | "st_w1_1640995200_100-500_200"  // 根据空间信息调整
        ├── temporal_bounds: {start: 1640995200000, end: 1640995260000}  // 必需
        ├── spatial_bounds: {min_x: 100, min_y: 200, max_x: 500, max_y: 200} | null  // 可选
        ├── packet_count: 1250
        ├── quality_metrics: {
            ├── path_completeness: 0.98        // 必需：逻辑路径完整性
            ├── spatial_coverage: 1.0 | null   // 可选：空间覆盖率
            └── temporal_continuity: 0.95      // 必需：时间连续性
        }
        └── spatial_hops: [
            ├── {
                ├── logical_index: 0           // 必需：逻辑跳序号
                ├── switch_id: "s1"            // 必需：交换机ID
                ├── coordinates: {x: 100, y: 200, z: 0} | null  // 可选：物理坐标
                ├── neighborhood: ["s2"] | null // 可选：邻居节点
                ├── temporal_samples: [        // 必需：时序数据
                    ├── {timestamp: 1640995201000, metrics: {delay: 200ns, queue: 0.8}}
                    ├── {timestamp: 1640995202000, metrics: {delay: 250ns, queue: 0.85}}
                    └── ...
                ]
                └── aggregated_metrics: {
                    ├── avg_delay: 225ns       // 必需：基础统计
                    ├── max_queue: 0.9         // 必需：基础统计
                    ├── spatial_gradient: {dx_delay: 50ns/hop, dy_delay: 0ns/hop} | null  // 可选：空间梯度
                    └── temporal_trend: "increasing"  // 必需：时间趋势
                }
            }
        ]
    }
]
└── spatiotemporal_indices: {
    ├── rtree_index: "spatial_index_id_12345" | null     // 可选：仅当有空间信息时
    ├── temporal_btree: "time_index_id_67890"            // 必需：时间索引
    ├── st_compound_index: "st_index_id_24680" | null    // 可选：仅当有空间信息时
    └── logical_path_trie: "path_index_id_13579"         // 必需：逻辑路径索引
}
```

#### 功能降级策略 (当无空间信息时)
```
无空间信息模式:
├── 索引策略降级:
    ├── 禁用 R-Tree 空间索引
    ├── 禁用 ST-Tree 复合索引  
    ├── 保留 B+Tree 时间索引
    └── 保留 Trie 逻辑路径索引
├── 查询功能降级:
    ├── 支持: 时间范围查询、逻辑路径查询
    ├── 支持: 跳间关联、路径完整性分析
    ├── 不支持: 空间范围查询、邻近查询
    └── 不支持: 空间几何分析、物理拓扑查询
├── 存储优化:
    ├── 空间字段置为 null，节省存储空间
    ├── 窗口ID简化为逻辑格式
    └── 跳过空间相关的聚合计算
└── 后续升级路径:
    ├── 用户可随时添加空间坐标信息
    ├── 系统自动启用空间索引和查询
    └── 支持渐进式功能扩展
```

#### 配置示例
```yaml
# 仅逻辑路径模式 (入门级配置)
intdb_config:
  spatial_mode: "logical_only"
  required_fields: ["flow_id", "logical_path", "temporal_data"]
  optional_fields: ["spatial_coordinates", "topology_info"]
  
# 完整时空模式 (高级配置)  
intdb_config:
  spatial_mode: "full_spatiotemporal"
  topology_source: "network_discovery" | "manual_config" | "import_from_sdn"
  coordinate_system: "datacenter_rack" | "geographic_gps" | "logical_grid"
```

#### 时空索引策略
```
多维索引结构:
├── 时间维度索引 (B+ Tree):
    ├── 主键: timestamp
    ├── 叶节点: 指向spatiotemporal_windows
    └── 范围查询优化: O(log n + k)
├── 空间维度索引 (R-Tree):
    ├── 空间范围: (min_x, min_y, max_x, max_y)
    ├── 叶节点: 指向spatial_hops
    └── 邻近查询优化: O(log n + k)
├── 路径维度索引 (Trie):
    ├── 路径前缀: s1 → s1->s2 → s1->s2->s3
    ├── 叶节点: 指向相同路径的flows
    └── 路径匹配查询: O(path_length)
└── 复合时空索引 (ST-Tree):
    ├── 同时索引时间和空间维度
    ├── 支持时空范围查询
    └── 最优化时空关联查询: O(log n + k)
```

#### 时空查询示例
```sql
-- 时空范围查询
SELECT flow_id, avg_delay 
FROM flows 
WHERE spatial_bounds INTERSECTS POLYGON((100 100, 600 100, 600 300, 100 300))
  AND temporal_bounds OVERLAPS TIMERANGE('2025-01-01T10:00:00Z', '2025-01-01T11:00:00Z')
  AND path_contains(['s1', 's2']);

-- 空间邻近查询  
SELECT * FROM flows
WHERE ST_Distance(path_geometry, POINT(300, 200)) < 100
  AND timestamp > NOW() - INTERVAL '1 hour';

-- 时空轨迹查询
SELECT flow_id, ST_AsText(path_geometry), temporal_bounds
FROM flows
WHERE ST_Intersects(path_geometry, LINESTRING(0 0, 1000 1000))
  ORDER BY creation_time;
```

#### 时间窗口分片策略
```
时间分片规则:
├── 默认窗口大小: 60秒 (可配置)
├── 活跃流: 保持最近3个窗口在内存
├── 历史窗口: 压缩后存储到磁盘
└── 窗口切换: 软切换，允许重叠缓冲

内存管理:
├── MemTable: 活跃流的当前窗口
├── ImmutableMemTable: 正在刷盘的窗口
└── L0-Ln SST: 分层存储历史窗口
```

#### 增量更新机制
```
连续遥测处理:
├── 包到达 → 定位流和窗口 → 增量更新metrics
├── 乱序处理 → 时间戳检查 → 插入到正确窗口
├── 缺失检测 → 超时窗口 → 标记不完整路径
└── 流结束 → 状态更新 → 触发压缩存储

实时聚合:
├── 滑动窗口统计: min/max/avg/p99延迟
├── 路径质量评分: 基于丢包率和延迟
└── 异常检测标志: 突发延迟、路径变化
```

### 复杂场景处理示例

**路径变化处理**:
```json
{
  "flow_id": "17343111537",
  "telemetry": [
    {"switch_id": "s1", "timestamp": "2025-04-21T10:00:00Z"},
    {"switch_id": "s2", "timestamp": "2025-04-21T10:00:01Z"},
    {"switch_id": "s4", "timestamp": "2025-04-21T10:00:02Z"}
  ],
  "path_change": {
    "reason": "link_failure",
    "original_path": ["s1", "s2", "s3"],
    "new_path": ["s1", "s2", "s4"]
  }
}
```

**部分路径缺失处理**:
```json
{
  "flow_id": "17343111538", 
  "telemetry": [
    {"switch_id": "s1", "timestamp": "2025-04-21T10:00:00Z"},
    {"switch_id": "s3", "timestamp": "2025-04-21T10:00:02Z"}
  ],
  "missing_hops": ["s2"],
  "reason": "switch_overload_or_failure"
}
```

## 核心差异化优势

### 🎯 路径语义原生支持
```sql
-- IntDB: 原生路径查询
SELECT flow_id, avg_delay, max_queue_util
FROM flows 
WHERE path_contains(['s1', 's2', 's3'])
  AND start_time > '2025-01-01T00:00:00Z';

-- InfluxDB: 需要复杂JOIN
SELECT DISTINCT flow_id FROM (
  SELECT flow_id FROM telemetry WHERE switch='s1' 
  INTERSECT 
  SELECT flow_id FROM telemetry WHERE switch='s2'
  INTERSECT
  SELECT flow_id FROM telemetry WHERE switch='s3'
);
```

### 📦 存储效率优化
- **路径去重压缩**: 相同路径的流共享存储模板
- **跳序列优化**: 保持时空关系的紧凑存储
- **存储节省**: 40-60%（相比传统TSDB）

### ⚡ 查询性能特化
- **路径查询**: 5-10倍性能提升（O(log n) vs O(n)）
- **跳间关联**: 原生支持，无需应用层计算
- **实时流重组**: 智能缓冲和批量优化

## 系统架构概览

### 存储层次
```
┌─────────────────────────────────────────────────────────┐
│                   Query Layer                           │
├─────────────────────────────────────────────────────────┤
│  Path Query Engine  │  Time Index  │  Spatial Index    │
├─────────────────────────────────────────────────────────┤
│                  Storage Engine                         │
├──────────────────┬──────────────────┬───────────────────┤
│   MemTable       │    WAL           │   SST Files       │
│ (Active Flows)   │ (Persistence)    │ (Disk Storage)    │
└──────────────────┴──────────────────┴───────────────────┘
```

### 数据处理流程
```
INT Data → JSON Parser → Flow Reassembler → Path Indexer → Storage
    ↓             ↓             ↓              ↓           ↓
Raw Hops → Parsed Hops → Complete Flows → Indexed → Persisted
```

### 索引体系
- **路径索引**: 前缀树快速路径匹配
- **时间索引**: B+树支持时间范围查询  
- **空间索引**: 网格索引支持网络拓扑查询
- **指标索引**: 范围树支持条件过滤

## 适用场景

### ✅ IntDB最佳适用场景
- **数据中心网络监控**: 东西向流量分析
- **SD-WAN性能分析**: 跨节点路径质量评估  
- **网络故障诊断**: 端到端路径异常检测
- **流量工程优化**: 基于历史路径数据的路由优化
- **网络安全分析**: 异常路径和流量模式检测

### ⚠️ 不推荐使用IntDB的场景
- **传统服务器监控**: CPU、内存、磁盘等单点指标
- **应用性能监控**: 事务响应时间、QPS等标量指标
- **IoT传感器数据**: 温度、湿度等简单时序数据
- **金融时序分析**: 股价、交易量等传统时序场景

## 部署与使用

### 🚀 快速开始

IntDB支持多平台部署，请根据您的操作系统选择对应的部署指南：

#### 📖 平台专用部署指南

| 平台 | 部署指南 | 推荐方法 |
|------|----------|----------|
| **🐧 Linux** | [📋 LINUX_DEPLOYMENT.md](./LINUX_DEPLOYMENT.md) | Docker部署、自动安装脚本 |
| **🍎 macOS** | [📋 MACOS_DEPLOYMENT.md](./MACOS_DEPLOYMENT.md) | 直接编译、Docker运行 |
| **🪟 Windows** | 即将支持 | WSL + Linux方法 |

#### ⚡ 一分钟快速体验

```bash
# 1. 克隆项目
git clone https://github.com/lzhtan/IntDB.git
cd IntDB

# 2. 启动测试服务器
cargo run --example test_api_server

# 3. 验证运行（新开终端）
curl http://127.0.0.1:3000/health
```

### 🔧 API使用示例

#### 基础操作
```bash
# 健康检查
curl http://127.0.0.1:2999/health
# 响应: {"status":"healthy","version":"0.2.0","uptime_seconds":5,"flow_count":3}

# 获取统计信息  
curl http://127.0.0.1:2999/stats

# 查询特定流
curl http://127.0.0.1:2999/flows/test_flow_1
```

#### 数据写入

**手动数据写入**:
```bash
curl -X POST http://127.0.0.1:2999/flows \
  -H 'Content-Type: application/json' \
  -d '{
    "flow": {
      "path": ["s1", "s2", "s3"],
      "hops": [
        {
          "hop_index": 0,
          "switch_id": "s1", 
          "timestamp": "2025-06-06T10:00:00Z",
          "metrics": {
            "queue_util": 0.8,
            "delay_ns": 200,
            "bandwidth_bps": 1000
          }
        }
      ]
    }
  }'
```

#### 高级查询
```bash
# 路径查询
curl -X POST http://127.0.0.1:2999/query \
  -H 'Content-Type: application/json' \
  -d '{"path_conditions": [{"contains": ["s1", "s2"]}]}'

# 时间范围查询
curl -X POST http://127.0.0.1:2999/query \
  -H 'Content-Type: application/json' \
  -d '{"time_conditions": [{"after": "2025-01-01T00:00:00Z"}]}'

# 复合条件查询
curl -X POST http://127.0.0.1:2999/query \
  -H 'Content-Type: application/json' \
  -d '{
    "path_conditions": [{"through_switch": "s2"}],
    "metric_conditions": [{"total_delay_greater_than": 500}],
    "limit": 10
  }'
```

**使用遥测数据生成器**:

IntDB提供了多种数据生成和测试工具：

```bash
# 1. 确保IntDB服务运行
./target/release/intdb &

# 2. 实时遥测数据生成器 - 持续生成数据
python3 telemetry_generator.py

# 3. 批量性能测试工具 - 生成大量数据进行性能测试
python3 batch_telemetry_generator.py --records 1000 --iterations 10
```

#### 工具特性对比

**实时遥测生成器 (`telemetry_generator.py`)**:
- 🔄 **持续数据生成**: 每秒生成一次遥测测量数据
- 📊 **固定网络路径**: s1 → s2 → s3 → s4 (4跳路径)
- 📈 **真实指标变化**: 队列利用率和延迟随时间动态变化
- ✅ **数据追加机制**: 所有测量数据追加到同一流中，不会覆盖
- 📱 **实时状态显示**: 显示每个交换机的当前队列利用率和延迟

**批量性能测试工具 (`batch_telemetry_generator.py`)**:
- 🚀 **大规模数据生成**: 快速生成指定数量的网络流记录
- 📊 **多样化网络拓扑**: 4个spine交换机 + 8个leaf交换机 + 32个服务器
- ⚡ **性能基准测试**: 对IntDB和InfluxDB进行写入性能对比
- 🔍 **查询性能验证**: 路径模式匹配、路径聚合等复杂查询测试
- 📈 **统计分析**: 自动计算响应时间、成功率、P95延迟等关键指标
- 📋 **报告生成**: 生成JSON和Markdown格式的详细性能报告

#### 输出示例

**实时遥测生成器输出**:
```
============================================================
  IntDB Telemetry Data Generator Started
============================================================
Target: http://localhost:2999/flows
Flow ID: network_monitoring_flow (FIXED - all data appends to same flow)
Interval: 1 second
Press Ctrl+C to stop

[10:15:23] s1: q=0.12 d=145ns | s2: q=0.34 d=268ns | s3: q=0.56 d=412ns | s4: q=0.23 d=198ns ✅
[10:15:24] s1: q=0.15 d=156ns | s2: q=0.38 d=278ns | s3: q=0.52 d=398ns | s4: q=0.27 d=208ns ✅
```

**批量性能测试工具输出**:
```bash
# 基本使用
python3 batch_telemetry_generator.py --records 1000 --iterations 10

# 完整性能对比测试
python3 batch_telemetry_generator.py --records 5000 --iterations 20

# 仅生成数据不查询
python3 batch_telemetry_generator.py --data-only --records 1000

# 仅执行查询测试
python3 batch_telemetry_generator.py --query-only --iterations 5
```

输出包含：
- 📊 **写入性能统计**: 成功率、吞吐量、平均响应时间
- 🔍 **查询性能对比**: IntDB vs InfluxDB各类查询的响应时间
- 📈 **性能提升指标**: 具体的性能改进百分比
- 📋 **详细报告**: 自动生成`performance_test_report_[timestamp].json`和`.md`文件