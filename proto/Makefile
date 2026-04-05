SIM_PATH ?= ../mobile/simulator/SkjuSimulator/


.PHONY: all
all: ${SIM_PATH}heim.pb.swift ${SIM_PATH}heim.grpc.swift

${SIM_PATH}heim.pb.swift: heim.proto
	protoc heim.proto \
		--swift_out="${SIM_PATH}" \
		--swift_opt=Visibility=Internal

${SIM_PATH}heim.grpc.swift:
	protoc heim.proto \
		--grpc-swift-2_out="${SIM_PATH}". \
   		--grpc-swift-2_opt=Visibility=Internal

.PHONY: clean
clean:
	@unlink "${SIM_PATH}heim.grpc.swift" 2>/dev/null || true
	@unlink "${SIM_PATH}heim.pb.swift" 2>/dev/null || true
