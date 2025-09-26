#!/bin/bash

# Gold 留言板项目启动脚本

echo "=== Gold 留言板项目启动脚本 ==="
echo

# 检查 MySQL 是否运行
# echo "🔍 检查 MySQL 服务状态..."
# if ! command -v mysql &> /dev/null; then
#     echo "❌ MySQL 未安装，请先安装 MySQL"
#     echo "macOS: brew install mysql"
#     echo "Ubuntu: sudo apt install mysql-server"
#     exit 1
# fi

# 检查 MySQL 连接
# if ! mysql -h127.0.0.1 -uroot -p -e "SELECT 1;" &> /dev/null; then
#     echo "❌ 无法连接到 MySQL，请检查："
#     echo "1. MySQL 服务是否运行: brew services start mysql"
#     echo "2. 用户名和密码是否正确"
#     echo "3. .env 文件中的数据库连接配置"
#     exit 1
# fi

# echo "✅ MySQL 连接正常"

# 检查数据库是否存在
echo "🔍 检查数据库..."
if ! mysql -h127.0.0.1 -uroot -p -e "USE gold_messageboard;" &> /dev/null; then
    echo "📦 创建数据库 gold_messageboard..."
    mysql -h127.0.0.1 -uroot -p -e "CREATE DATABASE gold_messageboard;"
    echo "✅ 数据库创建成功"
fi

# 运行数据库迁移
echo "🔧 运行数据库迁移..."
mysql -h127.0.0.1 -uroot -pgold_messageboard < migrations/001_create_users.sql
mysql -h127.0.0.1 -uroot -pgold_messageboard < migrations/002_create_messages.sql
echo "✅ 数据库迁移完成"

echo
echo "🚀 启动服务器..."
echo "服务器将在 http://127.0.0.1:3000 启动"
echo "按 Ctrl+C 停止服务器"
echo

# 启动应用
cargo run