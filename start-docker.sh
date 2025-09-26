#!/bin/bash

# Docker 环境启动脚本

echo "=== 使用 Docker 启动 Gold 留言板项目 ==="
echo

# 检查 Docker 是否安装
if ! command -v docker &> /dev/null; then
    echo "❌ Docker 未安装，请先安装 Docker"
    echo "访问 https://docs.docker.com/get-docker/ 获取安装指南"
    exit 1
fi

echo "🐳 启动 MySQL Docker 容器..."
docker run -d \
    --name mysql-gold \
    -e MYSQL_ROOT_PASSWORD=password \
    -e MYSQL_DATABASE=gold_messageboard \
    -p 3306:3306 \
    mysql:8.0

echo "⏳ 等待 MySQL 启动（30秒）..."
sleep 30

echo "🔧 运行数据库迁移..."
docker exec -i mysql-gold mysql -uroot -ppassword gold_messageboard < migrations/001_create_users.sql
docker exec -i mysql-gold mysql -uroot -ppassword gold_messageboard < migrations/002_create_messages.sql

echo "✅ 数据库设置完成"
echo
echo "🚀 启动服务器..."
echo "服务器将在 http://127.0.0.1:3000 启动"
echo "按 Ctrl+C 停止服务器"
echo

# 启动应用
cargo run