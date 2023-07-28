.PHONY: all
all:
	git submodule init
	git submodule update
	hugo
