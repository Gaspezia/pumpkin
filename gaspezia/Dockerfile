# Image de base du serveur Pumpkin (fork Gaspezia). Runtime seul : le binaire est compile
# avant (Jenkinsfile ou gaspezia/build-local.sh) puis copie depuis le contexte de build.
#   contexte attendu : ./pumpkin (binaire release) + ce Dockerfile
FROM ubuntu:26.04
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY pumpkin /usr/local/bin/pumpkin
# uid 2613 comme upstream ; USER numerique (exige par runAsNonRoot). Pas de HEALTHCHECK : Agones gere la sante.
RUN groupadd -g 2613 pumpkin && \
    useradd -u 2613 -g pumpkin -m -d /pumpkin pumpkin && \
    chown -R pumpkin:pumpkin /pumpkin
WORKDIR /pumpkin
USER 2613:2613
ENV RUST_BACKTRACE=1
EXPOSE 25565
ENTRYPOINT ["pumpkin"]
