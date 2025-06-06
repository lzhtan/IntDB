# IntDB
IntDB是一个面向带内网络遥测的时空数据库。

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

### Linux环境快速部署

#### 方法1：Docker部署（推荐）
```bash
# 克隆项目
git clone https://github.com/lzhtan/IntDB.git
cd IntDB

# Docker Compose启动（包含监控栈）
docker-compose up -d

# 验证服务
curl http://localhost:3000/health
```

#### 方法2：编译部署
```bash
# 1. 安装Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. 克隆项目
git clone https://github.com/lzhtan/IntDB.git
cd IntDB

# 3. 编译
cargo build --release

# 4. 启动服务
./target/release/examples/api_server
```

#### 方法3：自动化安装脚本
```bash
# Ubuntu/Debian/CentOS/RHEL支持
sudo bash deploy/install.sh

# 启动systemd服务
sudo systemctl start intdb
sudo systemctl enable intdb
```

### API使用示例
```bash
# 健康检查
curl http://localhost:3000/health

# 写入INT数据
curl -X POST http://localhost:3000/flows \
  -H 'Content-Type: application/json' \
  -d '{"flow": {"path": ["s1", "s2", "s3"], "hops": [...]}}'

# 查询流数据
curl http://localhost:3000/flows/test_flow_1

# 高级查询
curl -X POST http://localhost:3000/query \
  -H 'Content-Type: application/json' \
  -d '{"path_conditions": [{"contains": ["s1", "s2"]}]}'
```

### 配置示例
```toml
# /etc/intdb/config.toml
[server]
bind = "127.0.0.1:8086"
log_level = "info"

[storage]
data_dir = "/var/lib/intdb"
wal_dir = "/var/lib/intdb/wal"
memory_limit = "8GB"

[indexing]
path_index_cache = "1GB"
time_index_cache = "512MB"
enable_adaptive_indexing = true

[performance]
batch_size = 10000
flush_interval = "5s"
compression = "snappy"
```

## 兼容性策略

### 平滑迁移支持
```bash
# 支持InfluxDB Line Protocol写入
curl -XPOST 'http://localhost:8086/write?db=intdb' \
  --data-binary 'telemetry,flow=123,switch=s1 delay=500,queue_util=0.8'

# 提供InfluxQL兼容查询
SELECT mean(delay) FROM telemetry WHERE time > now() - 1h
```

### 生态系统集成
- **Telegraf适配器**: 复用现有数据收集工具
- **Grafana插件**: 可视化INT数据和路径分析
- **InfluxDB数据桥接**: 与现有TSDB协作部署

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

# 构建项目
cargo build

# 运行测试
cargo test

# 启动开发服务器
cargo run --example test_api_server
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
- 📖 部署文档：[DEPLOYMENT.md](./DEPLOYMENT.md)

---

**IntDB v0.1.0：专为网络遥测设计的时空数据库，填补INT数据管理的技术空白。**
