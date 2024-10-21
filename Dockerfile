# Etapa 1: Build
FROM rust:latest AS builder

WORKDIR /usr/src/backend-newsletter

# Copia os arquivos do projeto para o container
COPY . .

# Compila o projeto em modo release
RUN cargo build --release

# Etapa 2: Final
FROM archlinux:latest

WORKDIR /usr/src

# Atualiza e instala NGINX, git, vim e outras ferramentas necessárias
RUN pacman -Syu --noconfirm
RUN pacman -S --noconfirm git vim nginx postgresql-libs
RUN pacman -Scc --noconfirm

# Copie o binário do builder para a imagem final
COPY --from=builder /usr/src/backend-newsletter/target/release/backend-newsletter /usr/local/bin/backend-newsletter

# Copia o arquivo de configuração do NGINX
COPY nginx.conf /etc/nginx/nginx.conf

# Exponha a porta 3000 (da aplicação Rust) e a porta 80 (para o NGINX)
EXPOSE 3000
EXPOSE 80

# Inicia o NGINX e o binário Rust simultaneamente
CMD ["sh", "-c", "nginx && /usr/local/bin/backend-newsletter"]
