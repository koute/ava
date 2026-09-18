FROM ava/base

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# The installer only knows the home, so the binary moves into the image.
RUN curl -fsSL https://opencode.ai/install | VERSION=1.18.23 PATH="${AGENT_HOME}/.opencode/bin:${PATH}" bash && \
	sudo install -m 755 ${AGENT_HOME}/.opencode/bin/opencode /usr/local/bin/opencode && \
	rm -rf ${AGENT_HOME}/.opencode

RUN mkdir -p ${AGENT_HOME}/.config/opencode && \
	chown -R 1000:1000 ${AGENT_HOME}/.config

ENV npm_config_offline=true

ENTRYPOINT ["ava-bridge", "opencode"]
