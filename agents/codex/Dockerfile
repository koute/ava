FROM ava/base

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN sudo npm install -g --ignore-scripts --prefix /usr/local @openai/codex@0.152.0 && \
	sudo rm -rf /root/.npm

RUN mkdir -p ${AGENT_HOME}/.codex

ENTRYPOINT ["ava-bridge", "codex"]
