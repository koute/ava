FROM ava/base

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

RUN sudo npm install -g --ignore-scripts --prefix /usr/local @earendil-works/pi-coding-agent@0.84.3 && \
	sudo rm -rf /root/.npm

RUN mkdir -p ${AGENT_HOME}/.pi/agent

ENTRYPOINT ["ava-bridge", "pi"]
