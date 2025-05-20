#### **1. create 子命令**

**功能**：创建新检查点
**语法**：
`checkpointctl create [OPTIONS] <CONTAINER_ID>`

| 选项               | 类型     | 默认值 | 描述                         |
| ------------------ | -------- | ------ | ---------------------------- |
| `-i, --interval` | duration | -      | 定时创建间隔（如 30m, 1h）   |
| `-l, --label`    | string   | -      | 添加元数据标签（可多次使用） |
| `-p, --parent`   | string   | -      | 父检查点ID（增量模式）       |
| `--compression`  | enum     | zstd   | 压缩算法（none/gzip/zstd）   |
| `--pre-freeze`   | -        | -      | 创建前冻结容器进程           |
| `--memory-limit` | size     | 1G     | 内存快照上限（如 512MB）     |

**示例**：

```bash
checkpointctl create -i 1h --label=prod --parent=cp-123 mycontainer
```

---

#### **2. restore 子命令**

**功能**：从检查点恢复容器
**语法**：
`checkpointctl restore [OPTIONS] <CHECKPOINT_ID>`

| 选项                  | 类型   | 描述                                             |
| --------------------- | ------ | ------------------------------------------------ |
| `-t, --target-node` | string | 指定恢复的目标节点                               |
| `--parallel`        | int    | 并行恢复线程数（默认3）                          |
| `--validate`        | -      | 恢复前验证数据完整性                             |
| `--resume`          | -      | 自动恢复进程执行状态                             |
| `--network-remap`   | CIDR   | 重映射容器网络（如 10.0.0.0/24→192.168.0.0/24） |

**示例**：

```bash
checkpointctl restore --parallel=5 --network-remap=10.0.0.0/24:192.168.1.0/24 cp-20230518
```

---

#### **3. list 子命令**

**功能**：列出所有检查点
**语法**：
`checkpointctl list [OPTIONS] [CONTAINER_ID]`

| 选项             | 描述                                     |
| ---------------- | ---------------------------------------- |
| `-o, --output` | 输出格式（text/json/yaml）               |
| `--sort`       | 排序字段（time/size/labels）             |
| `-f, --filter` | 过滤表达式（如 "label=prod && age<24h"） |
| `--show-tree`  | 显示检查点依赖树                         |

**示例**：

```bash
checkpointctl list -o json --filter="size>100MB || label=critical"
```

---

#### **4. merge 子命令**

**功能**：合并多个检查点
**语法**：
`checkpointctl merge [OPTIONS] <CONTAINER_ID>`

| 选项                | 描述                                                                                                       |
| ------------------- | ---------------------------------------------------------------------------------------------------------- |
| `-s, --strategy`  | 合并策略：`<br>` - time-based（时间窗口）`<br>` - incremental（增量合并）`<br>` - tagged（标签保留） |
| `--keep-daily`    | 保留最近N天的每日检查点                                                                                    |
| `--keep-hourly`   | 保留最近N小时的检查点                                                                                      |
| `--retain-labels` | 保留指定标签的检查点                                                                                       |
| `--aggressive`    | 启用深度去重模式                                                                                           |
| `--dry-run`       | 模拟合并不实际执行                                                                                         |

**示例**：

```bash
checkpointctl merge -s time-based --keep-daily=7 --aggressive mycontainer
```

---

#### **5. prune 子命令**

**功能**：清理旧检查点
**语法**：
`checkpointctl prune [OPTIONS] <CONTAINER_ID>`

| 选项                 | 描述                     |
| -------------------- | ------------------------ |
| `--keep-latest`    | 保留最近N个检查点        |
| `--keep-days`      | 保留N天内的检查点        |
| `--exclude-labels` | 排除指定标签             |
| `--max-storage`    | 存储空间上限（如 100GB） |
| `--prune-dangling` | 清理无父节点的孤立检查点 |

**示例**：

```bash
checkpointctl prune --keep-latest=5 --max-storage=50GB myapp
```

---

#### **6. info 子命令**

**功能**：显示详细信息
**语法**：
`checkpointctl info <CHECKPOINT_ID>`

| 选项           | 描述                   |
| -------------- | ---------------------- |
| `--metadata` | 显示完整元数据         |
| `--diff`     | 与另一检查点的差异比较 |
| `--verify`   | 验证数据完整性         |
| `--export`   | 导出为可移植格式       |

**示例**：

```bash
checkpointctl info --diff=cp-20230518-1400 cp-20230518-1430
```

---

#### **7. automanage 子命令**

**功能**：自动管理守护进程
**语法**：
`checkpointctl automanage [OPTIONS]`

| 选项                 | 描述                         |
| -------------------- | ---------------------------- |
| `--check-interval` | 自动检查间隔（默认5m）       |
| `--cpu-threshold`  | 触发创建的CPU阈值（默认80%） |
| `--mem-threshold`  | 内存使用阈值（默认90%）      |
| `--schedule-file`  | 自定义策略配置文件路径       |
| `--daemonize`      | 以守护进程模式运行           |

**示例**：

```bash
checkpointctl automanage --cpu-threshold=70% --mem-threshold=85% --daemonize
```

---

### **全局选项**

| 选项                  | 描述                                          |
| --------------------- | --------------------------------------------- |
| `-c, --config`      | 指定配置文件路径（默认~/.checkpointctl.yaml） |
| `--log-level`       | 日志级别（debug/info/warn/error）             |
| `--storage-backend` | 存储后端（local/s3/nfs）                      |
| `--criu-path`       | 指定CRIU可执行文件路径                        |

---

### **配置文件示例**

```yaml
storage:
  backend: s3
  s3:
    bucket: my-checkpoints
    region: us-west-2
    access_key: AKIAXXX
    secret_key: xxxxx

automerge:
  strategy: time-based
  keep_daily: 7
  aggressive: true

logging:
  level: info
  max_size: 100MB
```

---

### **输入输出规范**

1. **时间格式**：ISO 8601扩展格式 `YYYY-MM-DDTHH:MM:SSZ`
2. **大小单位**：支持 B/KB/MB/GB（如 `--memory-limit=512MB`）
3. **过滤语法**：`field operator value` 组合，支持逻辑运算符：
   - 字段：`size`, `age`, `label`, `status`
   - 运算符：`=`, `>`, `<`, `!=`, `in`
   - 示例：`"label in (prod,staging) && age<48h"`

---

该规范支持以下典型工作流：

```bash
# 创建定时检查点
checkpointctl create -i 2h --label=daily myapp

# 自动合并策略
checkpointctl automanage --schedule-file=policy.yaml

# 灾难恢复流程
checkpointctl list --filter="label=golden" -o json | jq '.[-1].id' | xargs checkpointctl restore
```
