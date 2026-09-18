FROM ava/base

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

# The installer only knows the home, so the binary moves into the image.
RUN curl -fsSL https://claude.ai/install.sh | bash -s 2.1.247 && \
	sudo install -m 755 "$(readlink -f ${AGENT_HOME}/.local/bin/claude)" /usr/local/bin/claude && \
	rm -rf ${AGENT_HOME}/.local/share/claude ${AGENT_HOME}/.local/bin/claude ${AGENT_HOME}/.claude/downloads

ENTRYPOINT ["ava-bridge", "claude", "--dangerously-skip-permissions"]
