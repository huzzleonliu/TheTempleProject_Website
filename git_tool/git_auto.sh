#!/bin/bash

# 获取当前日期和时间，并格式化为 "年-月-日 时:分:秒" 的形式
date_time=$(date +"%Y-%m-%d %T")

# 添加所有修改到暂存区
git add ..

# 提交并添加日期作为提交信息
git commit -m "$date_time"

# 拉取最新代码
git push
