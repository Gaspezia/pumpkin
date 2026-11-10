// Build de l'image de base Pumpkin (fork Gaspezia). Seule la branche `gaspezia` porte ce fichier.
//   rust   : cargo build --release -p pumpkin --features plugin-v0_2
//   kaniko : image runtime (gaspezia/Dockerfile) -> docker-private-mc/pumpkin:<sha complet> + :gaspezia
// Si le build manque de memoire : gaspezia/build-local.sh (a noter dans le journal du pilote).
pipeline {
  agent {
    kubernetes {
      cloud 'k8s'
      defaultContainer 'jnlp'
      yaml '''
apiVersion: v1
kind: Pod
metadata:
  labels:
    app.kubernetes.io/component: jenkins-build
    app.kubernetes.io/part-of: pumpkin-image
spec:
  serviceAccountName: jenkins-agent
  # Nœud de build dédié (taint pool=ci) : c'est le seul à avoir ~10 Gi libres.
  nodeSelector:
    pool: ci
  tolerations:
    - { key: pool, operator: Equal, value: ci, effect: NoSchedule }
  containers:
    - name: jnlp
      image: nexus.gaspezia.lan/docker-private/jenkins-inbound-agent:gaspezia
      resources:
        requests: { cpu: "200m", memory: "256Mi" }
        limits:   { cpu: "2", memory: "512Mi" }
    - name: rust
      image: nexus.gaspezia.lan/docker-all/rust:1.98-trixie
      command: ["sleep", "infinity"]
      tty: true
      resources:
        # Mesure 2026-11-10 : lien final (LTO fat) = 9,9 Go de RSS pour un seul processus. A 8Gi : OOMKilled (builds #2 a #4).
        requests: { cpu: "2", memory: "8Gi", ephemeral-storage: "10Gi" }
        limits:   { cpu: "4", memory: "12Gi", ephemeral-storage: "16Gi" }
    - name: kaniko
      image: gcr.io/kaniko-project/executor:debug
      command: ["/busybox/cat"]
      tty: true
      resources:
        requests: { cpu: "500m", memory: "1Gi", ephemeral-storage: "4Gi" }
        limits:   { cpu: "2", memory: "4Gi", ephemeral-storage: "8Gi" }
      volumeMounts:
        - { name: nexus-docker-config, mountPath: /kaniko/.docker }
        - { name: gaspezia-ca, mountPath: /kaniko/ssl/certs/additional-ca-cert-bundle.crt, subPath: ca.crt }
  volumes:
    - name: nexus-docker-config
      secret: { secretName: nexus-ci, items: [{ key: .dockerconfigjson, path: config.json }] }
    - name: gaspezia-ca
      configMap: { name: gaspezia-ca, items: [{ key: ca.crt, path: ca.crt }] }
'''
    }
  }

  options { timestamps(); disableConcurrentBuilds() }

  environment {
    DOCKER_REGISTRY        = 'nexus.gaspezia.lan/docker-private-mc'
    DISCORD_CREDENTIALS_ID = 'discord-webhook'
  }

  stages {
    stage('Compiler') {
      when { branch 'gaspezia' }
      steps {
        container('rust') {
          sh 'cargo build --release -p pumpkin --features plugin-v0_2'
        }
      }
    }

    stage('Image (Kaniko)') {
      when { branch 'gaspezia' }
      steps {
        script {
          env.GIT_SHA = sh(script: 'git rev-parse HEAD', returnStdout: true).trim()
          env.IMAGE_SHA    = "${DOCKER_REGISTRY}/pumpkin:${env.GIT_SHA}"
          env.IMAGE_STABLE = "${DOCKER_REGISTRY}/pumpkin:gaspezia"
          sh 'rm -rf build-context && mkdir build-context && cp target/release/pumpkin gaspezia/Dockerfile build-context/'
          container('kaniko') {
            sh """
              /kaniko/executor \\
                --context dir://\$(pwd)/build-context \\
                --dockerfile build-context/Dockerfile \\
                --destination ${env.IMAGE_SHA} \\
                --destination ${env.IMAGE_STABLE}
            """
          }
        }
      }
    }
  }

  post {
    always {
      script {
        def result = currentBuild.currentResult
        def lines  = ["**Branch:** ${env.BRANCH_NAME}"]
        if (env.IMAGE_SHA) { lines << "**Image:** `${env.IMAGE_SHA}`" }
        def icon  = (result == 'SUCCESS') ? '✅' : '❌'
        def color = (result == 'SUCCESS') ? 3066993 : 15158332
        def payload = groovy.json.JsonOutput.toJson([embeds: [[
          title: "${icon} pumpkin (fork) — ${result}",
          description: lines.join("\n"), url: env.BUILD_URL, color: color,
          footer: [text: "Pipeline Notification"]]]])
        writeFile file: 'discord-payload.json', text: payload
        catchError(buildResult: 'UNSTABLE', stageResult: 'UNSTABLE') {
          withCredentials([string(credentialsId: env.DISCORD_CREDENTIALS_ID, variable: 'DISCORD_WEBHOOK')]) {
            sh '''
              set -eu
              curl --fail --silent --show-error --max-time 30 \
                -H "Content-Type: application/json" \
                --data-binary @discord-payload.json "$DISCORD_WEBHOOK"
            '''
          }
        }
        cleanWs()
      }
    }
  }
}
