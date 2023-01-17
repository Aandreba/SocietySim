build:
	glslangValidator --client vulkan100 -o target/main.spv src/main.comp

doc:
	cd vulkan && cargo rustdoc --open --all-features -- --cfg docsrs