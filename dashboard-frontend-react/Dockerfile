# 1. build 阶段：打包 Vite 前端
FROM node:20 AS build

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .

# 在浏览器里直接访问 http://localhost:3001，所以这里仍用这个地址
ENV VITE_API_BASE=http://127.0.0.1:3001

RUN npm run build

# 2. 运行阶段：静态文件用 nginx 服务
FROM nginx:alpine

# 把打包后的静态文件拷贝到 nginx 默认目录
COPY --from=build /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
