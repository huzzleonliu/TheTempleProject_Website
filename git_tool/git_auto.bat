
@echo off

REM 获取当前日期和时间，并格式化为 "年-月-日 时:分:秒" 的形式
for /f "tokens=2 delims==" %%I in ('wmic os get localdatetime /value') do set datetime=%%I
set datetime=%datetime:~0,4%-%datetime:~4,2%-%datetime:~6,2% %datetime:~8,2%:%datetime:~10,2%:%datetime:~12,2%

REM 添加所有修改到暂存区
git add .

REM 提交并添加日期作为提交信息
git commit -m "%datetime%"

REM 拉取最新代码
git push

pause