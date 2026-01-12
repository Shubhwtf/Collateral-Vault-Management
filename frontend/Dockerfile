FROM node:20-alpine as builder

RUN npm install -g pnpm

WORKDIR /app

COPY frontend/package.json frontend/pnpm-lock.yaml ./

RUN pnpm install --frozen-lockfile

COPY frontend/ .

RUN pnpm run build

FROM node:20-alpine

WORKDIR /app

RUN npm install -g serve

COPY --from=builder /app/dist ./dist

EXPOSE 3000

CMD ["serve", "-s", "dist", "-l", "3000"]
