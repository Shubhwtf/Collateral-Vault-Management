# Docker Deployment Guide

Complete guide for deploying the Collateral Vault Management System using Docker and Docker Compose.

## Prerequisites

- **Docker** 20.10 or later
- **Docker Compose** 2.0 or later
- **Solana CLI** 1.16+ (includes `spl-token` CLI) - for running `setup-solana-dev.sh`
- A Solana keypair file for the backend (can be generated with `setup-solana-dev.sh`)
- Environment variables configured (see Configuration section)

**Note:** For local development, run `./setup-solana-dev.sh` before starting Docker services to set up your Solana wallet and dev tokens.

## Quick Start

### Development

```bash
# 1. Set up Solana dev environment (first time only, or when you need new tokens)
chmod +x setup-solana-dev.sh
./setup-solana-dev.sh  # Or: ./setup-solana-dev.sh YOUR_PHANTOM_WALLET_PUBKEY

# 2. Start all services
docker-compose up -d

# 3. View logs
docker-compose logs -f
```

This starts all services:
- PostgreSQL database on port 5432
- Backend API on port 8080
- Frontend on port 3000

**Note:** Make sure to run `setup-solana-dev.sh` first to create your Solana wallet and mint dev tokens. The script output will show you the `USDT_MINT` address to use in your backend `.env` file.

### Production

```bash
docker-compose -f docker-compose.prod.yml up -d
```

## Configuration

### Environment Variables

Create a `.env` file in the project root with the following variables:

#### Required Variables

```bash
POSTGRES_PASSWORD=your_secure_password
PROGRAM_ID=your_solana_program_id
PAYER_KEYPAIR_PATH=/path/to/your/keypair.json
```

#### Optional Variables

```bash
POSTGRES_DB=collateral_vault
POSTGRES_USER=vault_user
POSTGRES_PORT=5432
BACKEND_PORT=8080
FRONTEND_PORT=3000

SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_WS_URL=wss://api.devnet.solana.com
USDT_MINT=Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB

RUST_LOG=info,vault_backend=debug
RUST_BACKTRACE=1
TRANSACTION_TIMEOUT_SECONDS=30
MAX_RETRY_ATTEMPTS=3

VITE_API_URL=http://localhost:8080
```

#### Production Variables

For production (`docker-compose.prod.yml`), also set:

```bash
REDIS_PASSWORD=your_redis_password
CORS_ALLOWED_ORIGINS=https://yourdomain.com
NGINX_HOST=yourdomain.com
VERSION=1.0.0
```

### Keypair Setup

The backend requires a Solana keypair file. Place your keypair file in a secure location and set `PAYER_KEYPAIR_PATH` to its absolute path.

**Security Note:** Never commit keypair files to version control. Use environment variables or Docker secrets in production.

### Development Token Setup

For local development and testing, use the `setup-solana-dev.sh` script to set up your Solana environment:

```bash
# Make script executable (first time only)
chmod +x setup-solana-dev.sh

# Set up dev environment (creates wallet, mints dev USDT tokens)
./setup-solana-dev.sh

# Or mint tokens to a specific wallet (e.g., Phantom for frontend testing)
./setup-solana-dev.sh YOUR_PHANTOM_WALLET_PUBKEY
```

**What the script does:**
- Creates or uses a Solana wallet keypair (default: `~/.config/solana/id.json`)
- Airdrops SOL if balance is low (< 1 SOL)
- Creates a dev USDT token mint (stored in `.dev/dev-usdt-mint-keypair.json`)
- Creates an associated token account for the target wallet
- Mints dev USDT tokens (default: 1,000,000 tokens with 6 decimals)

**Environment Variables:**
The script supports these optional environment variables:
- `SOLANA_RPC_URL` - RPC endpoint (default: `https://api.devnet.solana.com`)
- `SOLANA_KEYPAIR_PATH` - Payer wallet keypair (default: `~/.config/solana/id.json`)
- `DEV_USDT_MINT_KEYPAIR` - Mint keypair path (default: `./.dev/dev-usdt-mint-keypair.json`)
- `DEV_USDT_MINT_AMOUNT` - Amount to mint in base units (default: `1000000000000`)
- `DEV_SOL_AIRDROP_AMOUNT` - SOL to airdrop if needed (default: `2`)

**After running the script:**
1. Use the mint address output as `USDT_MINT` in your backend `.env`
2. Use the payer wallet path as `PAYER_KEYPAIR_PATH` in your backend `.env`
3. Your Phantom wallet (if specified) will have dev USDT tokens for frontend testing

## Services

### PostgreSQL

**Image:** `postgres:16-alpine`

**Features:**
- Automatic migration execution on startup
- Persistent data volume
- Health checks
- Default database: `collateral_vault`

**Data Persistence:**
- Volume: `vault_postgres_data` (development)
- Volume: `vault_postgres_data_prod` (production)

**Ports:**
- Development: `5432:5432` (exposed to host)
- Production: `127.0.0.1:5432:5432` (localhost only)

### Backend

**Image:** Built from `backend/Dockerfile`

**Features:**
- Rust backend compiled in multi-stage build
- Runs as non-root user (appuser)
- Health check endpoint at `/health`
- Automatic database connection retry
- Rate limiting support (Redis optional)

**Ports:**
- Development: `8080:8080`
- Production: `127.0.0.1:8080:8080` (localhost only, use reverse proxy)

**Volumes:**
- `./backend/migrations:/app/migrations:ro` - Migration files
- Keypair file mounted from host

**Environment:**
- All configuration via environment variables
- See `backend/src/config.rs` for required variables

### Frontend

**Image:** Built from `frontend/Dockerfile`

**Features:**
- Multi-stage build (Node.js builder, serve runtime)
- Static files served with `serve`
- Environment variables baked at build time

**Ports:**
- Development: `3000:80`
- Production: `80:80` and `443:443` (if SSL configured)

**Build Args:**
- `NODE_VERSION=20`
- `VITE_API_URL` - Backend API URL

**Note:** For production, consider using nginx instead of `serve` for better performance and SSL support.

### Redis (Production Only)

**Image:** `redis:7-alpine`

**Features:**
- AOF persistence enabled
- Password protected
- Used for distributed rate limiting

**Ports:**
- `127.0.0.1:6379:6379` (localhost only)

**Data Persistence:**
- Volume: `vault_redis_data_prod`

## Docker Compose Files

### docker-compose.yml (Development)

Use this for local development and testing.

**Features:**
- Exposed ports for easy access
- Debug logging enabled
- Automatic restart on failure
- Health checks for service dependencies

**Usage:**
```bash
docker-compose up -d
docker-compose logs -f
docker-compose down
```

### docker-compose.prod.yml (Production)

Use this for production deployments.

**Features:**
- Services only exposed to localhost (use reverse proxy)
- Optimized PostgreSQL configuration
- Log rotation configured
- Security hardening (read-only filesystem, no-new-privileges)
- Redis for distributed rate limiting
- Production logging levels

**Usage:**
```bash
docker-compose -f docker-compose.prod.yml up -d
docker-compose -f docker-compose.prod.yml logs -f
docker-compose -f docker-compose.prod.yml down
```

## Common Operations

### Start Services

```bash
docker-compose up -d
```

### Stop Services

```bash
docker-compose down
```

### View Logs

```bash
docker-compose logs -f
docker-compose logs -f backend
docker-compose logs -f postgres
```

### Restart a Service

```bash
docker-compose restart backend
```

### Execute Commands in Container

```bash
docker-compose exec backend /bin/sh
docker-compose exec postgres psql -U vault_user -d collateral_vault
```

### Rebuild After Code Changes

```bash
docker-compose build backend
docker-compose up -d --force-recreate backend
```

### View Service Status

```bash
docker-compose ps
```

### Check Health

```bash
curl http://localhost:8080/health
curl http://localhost:3000
```

## Database Management

### Run Migrations Manually

Migrations run automatically on PostgreSQL startup, but you can run them manually:

```bash
docker-compose exec postgres psql -U vault_user -d collateral_vault -f /docker-entrypoint-initdb.d/001_initial_schema.sql
```

### Backup Database

```bash
docker-compose exec postgres pg_dump -U vault_user collateral_vault > backup.sql
```

### Restore Database

```bash
docker-compose exec -T postgres psql -U vault_user collateral_vault < backup.sql
```

### Access Database Console

```bash
docker-compose exec postgres psql -U vault_user -d collateral_vault
```

## Troubleshooting

### Backend Won't Start

1. **Check logs:**
   ```bash
   docker-compose logs backend
   ```

2. **Verify environment variables:**
   ```bash
   docker-compose exec backend env | grep -E "PROGRAM_ID|DATABASE_URL|PAYER_KEYPAIR"
   ```

3. **Check keypair file:**
   - Ensure `PAYER_KEYPAIR_PATH` points to a valid file
   - Verify file permissions allow reading

4. **Database connection issues:**
   - Wait for PostgreSQL to be healthy: `docker-compose ps postgres`
   - Check database URL format
   - Verify PostgreSQL credentials

### Frontend Can't Connect to Backend

1. **Check VITE_API_URL:**
   - Must match backend URL
   - For Docker: use `http://backend:8080` or `http://localhost:8080`

2. **Verify network:**
   ```bash
   docker-compose exec frontend ping backend
   ```

3. **Check CORS settings** (if accessing from browser)

### Database Migrations Not Running

1. **Check migration files:**
   ```bash
   ls -la backend/migrations/
   ```

2. **Verify volume mount:**
   ```bash
   docker-compose exec postgres ls -la /docker-entrypoint-initdb.d/
   ```

3. **Check PostgreSQL logs:**
   ```bash
   docker-compose logs postgres | grep -i migration
   ```

### Port Already in Use

If ports are already in use, change them in `.env`:

```bash
POSTGRES_PORT=5433
BACKEND_PORT=8081
FRONTEND_PORT=3001
```

### Volume Permission Issues

If you encounter permission issues with volumes:

```bash
sudo chown -R $USER:$USER postgres_data
```

Or adjust volume paths in docker-compose.yml.

## Production Deployment

### Pre-Deployment Checklist

- [ ] Set strong passwords for PostgreSQL and Redis
- [ ] Configure production Solana RPC endpoint
- [ ] Set correct `PROGRAM_ID` and `USDT_MINT` for mainnet
- [ ] Configure reverse proxy (nginx/traefik) for SSL
- [ ] Set up log aggregation
- [ ] Configure monitoring and alerting
- [ ] Set up database backups
- [ ] Review security settings
- [ ] Test health checks
- [ ] Verify rate limiting configuration

### Reverse Proxy Setup

The production compose file exposes services only to localhost. Use a reverse proxy:

**Nginx Example:**
```nginx
server {
    listen 80;
    server_name api.yourdomain.com;
    
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### SSL/TLS Configuration

1. Obtain SSL certificates (Let's Encrypt, etc.)
2. Mount certificates to frontend container:
   ```yaml
   volumes:
     - ./ssl:/etc/nginx/ssl:ro
   ```
3. Configure nginx for HTTPS

### Monitoring

**Health Checks:**
- Backend: `http://localhost:8080/health`
- Frontend: `http://localhost/`
- PostgreSQL: Automatic via `pg_isready`
- Redis: Automatic via `redis-cli ping`

**Logs:**
```bash
docker-compose -f docker-compose.prod.yml logs -f --tail=100
```

### Scaling

For horizontal scaling:

1. **Backend:** Use a load balancer (nginx, traefik) with multiple backend instances
2. **Database:** Use connection pooling (already configured)
3. **Redis:** Use Redis Cluster for distributed rate limiting

### Backup Strategy

**Database Backups:**
```bash
docker-compose -f docker-compose.prod.yml exec postgres pg_dump -U vault_user collateral_vault | gzip > backup_$(date +%Y%m%d).sql.gz
```

**Automated Backups:**
Set up a cron job or use a backup service to run backups regularly.

## Security Considerations

1. **Never commit `.env` files** to version control
2. **Use Docker secrets** for sensitive data in production
3. **Restrict network access** - production services only expose to localhost
4. **Use strong passwords** for PostgreSQL and Redis
5. **Keep images updated** - regularly rebuild with latest base images
6. **Review security settings** - read-only filesystem, no-new-privileges
7. **Monitor logs** for suspicious activity
8. **Use SSL/TLS** for all external connections

## Performance Tuning

### PostgreSQL (Production)

The production compose file includes optimized PostgreSQL settings:
- `max_connections=100`
- `shared_buffers=256MB`
- `effective_cache_size=1GB`
- `work_mem=2621kB`

Adjust based on your server resources.

### Backend

- Connection pooling: 5 max connections (configured in code)
- Rate limiting: Configured per endpoint tier
- Logging: Adjust `RUST_LOG` for production

### Frontend

- Consider using nginx instead of `serve` for better performance
- Enable gzip compression
- Configure caching headers

## Cleanup

### Remove All Containers and Volumes

```bash
docker-compose down -v
```

### Remove Images

```bash
docker-compose down --rmi all
```

### Complete Cleanup

```bash
docker-compose down -v --rmi all
docker system prune -a
```

**Warning:** This removes all data. Backup first!

## Additional Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [PostgreSQL Docker Image](https://hub.docker.com/_/postgres)
- [Redis Docker Image](https://hub.docker.com/_/redis)

