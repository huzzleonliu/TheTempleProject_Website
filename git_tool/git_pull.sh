
#!/bin/bash

git pull

echo "Press any key to exit..."
# 使用 read 命令等待用户按下任意键
read -n 1 -s  # -n 1 表示读取一个字符，-s 表示不显示输入字符
echo "Exiting..."