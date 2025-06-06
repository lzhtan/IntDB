# IntDB
IntDB是一个面向带内网络遥测的时空数据库。

> 🚀 **快速部署**：[Linux服务器部署](./LINUX_DEPLOYMENT.md) | [macOS本地开发](./MACOS_DEPLOYMENT.md)

## 📖 文档导航

| 内容 | 链接 | 说明 |
|------|------|------|
| **🛠️ Linux部署** | [LINUX_DEPLOYMENT.md](./LINUX_DEPLOYMENT.md) | Ubuntu/CentOS环境部署 |
| **🍎 macOS部署** | [MACOS_DEPLOYMENT.md](./MACOS_DEPLOYMENT.md) | macOS环境搭建 |
| **🐳 Docker部署** | [docker-compose.yml](./docker-compose.yml) | 容器化一键部署 |
| **📊 性能测试** | [PerformanceTestResult.md](./PerformanceTestResult.md) | 与InfluxDB的性能对比测试 |
| **💻 API示例** | [examples/](./examples/) | 代码示例和演示 |

## 设计理念与定位

IntDB**不是**传统时序数据库的替代品，而是专门为**带内网络遥测**场景设计的时空数据库。我们的核心理念是：

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
```
Flow Record:
├── flow_id: "17343111536"
├── path_hash: "sha256(s1->s2->s3)"
├── switch_sequence: ["s1", "s2", "s3"]
├── time_range: (start_time, end_time)
└── hops: [
    ├── {hop_idx: 0, switch: "s1", metrics: {...}}
    ├── {hop_idx: 1, switch: "s2", metrics: {...}}
    └── {hop_idx: 2, switch: "s3", metrics: {...}}
]
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
curl http://127.0.0.1:3000/health
# 响应: {"status":"healthy","version":"0.1.0","uptime_seconds":5,"flow_count":3}

# 获取统计信息  
curl http://127.0.0.1:3000/stats

# 查询特定流
curl http://127.0.0.1:3000/flows/test_flow_1
```

#### 数据写入
```bash
curl -X POST http://127.0.0.1:3000/flows \
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
curl -X POST http://127.0.0.1:3000/query \
  -H 'Content-Type: application/json' \
  -d '{"path_conditions": [{"contains": ["s1", "s2"]}]}'

# 时间范围查询
curl -X POST http://127.0.0.1:3000/query \
  -H 'Content-Type: application/json' \
  -d '{"time_conditions": [{"after": "2025-01-01T00:00:00Z"}]}'

# 复合条件查询
curl -X POST http://127.0.0.1:3000/query \
  -H 'Content-Type: application/json' \
  -d '{
    "path_conditions": [{"through_switch": "s2"}],
    "metric_conditions": [{"total_delay_greater_than": 500}],
    "limit": 10
  }'
```

### 配置示例

当前v0.1.0版本使用内存存储，以下为未来版本的配置规划：

```toml
# 未来版本配置示例
[server]
bind = "127.0.0.1:3000"  # 当前默认端口
log_level = "info"

[storage]
# v0.1.0: 纯内存存储，以下为未来规划
data_dir = "/var/lib/intdb"      # 持久化数据目录
wal_dir = "/var/lib/intdb/wal"   # 写前日志目录
memory_limit = "8GB"             # 内存使用限制

[indexing]
# v0.1.0: 内存索引，以下为未来优化
path_index_cache = "1GB"
time_index_cache = "512MB"
enable_adaptive_indexing = true

[performance]
max_flows = 1000000              # 当前支持：最大流数量
auto_cleanup_hours = 24          # 当前支持：自动清理
```

## 生态系统集成

### 当前集成状态
- ✅ **HTTP RESTful API**: 标准化接口，易于集成
- ✅ **JSON数据格式**: 通用格式，工具链友好
- ✅ **Docker支持**: 容器化部署，云原生兼容

### 未来兼容性规划
- 🔄 **InfluxDB Line Protocol**: 平滑迁移现有监控数据
- 🔄 **Grafana插件**: 专门的INT数据可视化
- 🔄 **Telegraf适配器**: 复用现有数据收集工具
- 🔄 **Prometheus集成**: 指标暴露和监控

## 性能预期

| 指标 | IntDB目标 | InfluxDB基准 | 说明 |
|------|-----------|--------------|------|
| **路径查询延迟** | 50-200ms | 500-2000ms | 5-10倍提升 |
| **简单查询延迟** | 50-150ms | 10-50ms | 2-5倍劣化 |
| **写入吞吐量** | 500K-1M/sec | 800K-1.2M/sec | 基本相当 |
| **存储效率** | 节省40-60% | 基准 | 显著优势 |
| **内存使用** | 减少30-50% | 基准 | 路径压缩 |

### 性能优势来源分析

**路径查询优化原理**:
- **索引优势**: 路径前缀树将O(n)扫描降为O(log n)查找
- **数据局部性**: 相同路径的流聚集存储，减少磁盘I/O
- **查询优化**: 路径谓词前置，大幅减少候选数据集

**存储效率提升机制**:
- **路径模板共享**: 相同拓扑路径只存储一次元数据
- **跳序列压缩**: 利用网络拓扑规律性进行差分编码  
- **时间聚合**: 批量写入时的时间戳压缩

**简单查询劣化原因**:
- **索引开销**: 维护多维索引增加查询成本
- **数据分散**: 非路径查询可能跨多个存储分区
- **优化偏向**: 引擎针对路径查询优化，牺牲部分通用性能

## 技术栈选择

- **实现语言**: Rust（完整实现，包含核心引擎和API层）
- **存储引擎**: 自定义存储引擎，专门针对路径时序数据优化
- **异步运行时**: Tokio（高性能异步I/O）
- **HTTP框架**: Axum（快速、安全的Web框架）
- **索引系统**: 多维索引（路径前缀树、时间B+树、指标范围树）
- **序列化**: Serde（类型安全的JSON处理）
- **查询引擎**: 路径感知的查询优化器


## 项目状态

### 🎉 IntDB v0.1.0 已发布

**核心功能完成度**：
- ✅ INT数据模型（Flow、Hop、TelemetryMetrics、NetworkPath）
- ✅ 存储引擎（PathIndex、TimeIndex、StorageEngine）
- ✅ 查询系统（路径查询、时间查询、指标过滤、复合查询）
- ✅ HTTP RESTful API（健康检查、CRUD、高级查询）
- ✅ Linux部署支持（Docker、systemd、自动安装脚本）
- ✅ 测试验证（28个单元测试全部通过）

**生产就绪性**：适用于开发和测试环境，具备完整的INT数据管理能力。

## 开发贡献

### 参与开发
```bash
# 克隆项目
git clone https://github.com/lzhtan/IntDB.git
cd IntDB

# 运行测试
cargo test

# 启动开发服务器（包含测试数据）
cargo run --example test_api_server

# 代码检查和修复
cargo clippy
cargo fmt
```

### 代码贡献
1. Fork本仓库
2. 创建功能分支：`git checkout -b feature/your-feature`
3. 提交代码：`git commit -am 'Add some feature'`
4. 推送分支：`git push origin feature/your-feature`
5. 提交Pull Request

### 社区
- 🐛 问题反馈：[GitHub Issues](https://github.com/lzhtan/IntDB/issues)
- 💬 功能讨论：[GitHub Discussions](https://github.com/lzhtan/IntDB/discussions)
- 📖 部署文档：[Linux部署](./LINUX_DEPLOYMENT.md) | [macOS部署](./MACOS_DEPLOYMENT.md)

---

**IntDB：专为网络遥测设计的时空数据库。**
