bin_dir := bin
third_party_dir := third_party
ar = ${WASI_SDK_PATH}/bin/ar
cc = ${WASI_SDK_PATH}/bin/clang

all: archive

archive: wasi_ext_lib.o json.o wasi-sdk-path
	$(ar) rcs $(bin_dir)/libwasi_ext_lib.a $(bin_dir)/wasi_ext_lib.o $(bin_dir)/json.o

wasi_ext_lib.o: $(bin_dir) wasi-sdk-path
	$(cc) $(CFLAGS) -I $(third_party_dir) -O -c wasi_ext_lib.c -o $(bin_dir)/wasi_ext_lib.o

json.o: $(bin_dir) wasi-sdk-path
	$(cc) $(CFLAGS) -O -c $(third_party_dir)/json/json.c -o $(bin_dir)/json.o

clean: $(bin_dir)
	rm -rf $(bin_dir)

$(bin_dir):
	mkdir $(bin_dir)

wasi-sdk-path:
ifndef WASI_SDK_PATH
	$(error WASI_SDK_PATH is undefined)
endif
