# 在8个核心上随机森林模型的训练计时的例子
from time import time
from sklearn.datasets import make_classification
from sklearn.ensemble import RandomForestClassifier
import sys

# 同时重定向 stdout 和 stderr 到文件
original_stdout = sys.stdout
original_stderr = sys.stderr
with open('verbose.log', 'w', buffering=1) as f:
    sys.stdout = f  # 重定向标准输出
    sys.stderr = f  # 重定向标准错误（verbose 输出通常在这里）
    
    # 定义数据集
    X, y = make_classification(n_samples=100000, n_features=20, 
                              n_informative=15, n_redundant=5, 
                              random_state=3, n_classes=2)
    # 定义模型
    model = RandomForestClassifier(n_estimators=500, n_jobs=8, verbose=1)
    
    # 记录当前时间
    start = time()
    # 训练模型
    model.fit(X, y)
    # 记录当前时间
    end = time()
    
    # 报告执行时间
    result = end - start
    print('%.3f seconds' % result)  # 这个仍然在控制台显示

    # 恢复标准输出和错误流
    sys.stdout = original_stdout
    sys.stderr = original_stderr


