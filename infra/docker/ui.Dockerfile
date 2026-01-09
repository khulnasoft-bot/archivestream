FROM node:20-alpine AS builder
WORKDIR /app/apps/web-ui
COPY apps/web-ui/package*.json ./
RUN npm install
COPY apps/web-ui .
RUN npm run build

FROM node:20-alpine AS runner
WORKDIR /app/apps/web-ui
COPY --from=builder /app/apps/web-ui/.next ./.next
COPY --from=builder /app/apps/web-ui/public ./public
COPY --from=builder /app/apps/web-ui/package*.json ./
RUN npm install --production
EXPOSE 3000
CMD ["npm", "start"]
