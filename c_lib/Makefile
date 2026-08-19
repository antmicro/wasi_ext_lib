# SPDX-License-Identifier: Apache-2.0
bin_dir := bin
third_party_dir := third_party
ar = ${WASI_SDK_PATH}/bin/ar
cc = ${WASI_SDK_PATH}/bin/clang

all: wasi_ext_lib c_sup

wasi_ext_lib: wasi_ext_lib.o json.o termios.o wasi-sdk-path
	$(ar) rcs $(bin_dir)/libwasi_ext_lib.a $(bin_dir)/wasi_ext_lib.o $(bin_dir)/json.o $(bin_dir)/termios.o

wasi_ext_lib.o: $(bin_dir) wasi-sdk-path
	$(cc) $(CFLAGS) -I $(third_party_dir) -O -c wasi_ext_lib.c -o $(bin_dir)/wasi_ext_lib.o

json.o: $(bin_dir) wasi-sdk-path
	$(cc) $(CFLAGS) -O -c $(third_party_dir)/json/json.c -o $(bin_dir)/json.o

termios.o: $(bin_dir) wasi-sdk-path
	$(cc) $(CFLAGS) -I . -I $(third_party_dir) -O -c $(third_party_dir)/termios/termios.c -o $(bin_dir)/termios.o

c_sup: wasi_c_sup.o wasi-sdk-path
	$(ar) rcs $(bin_dir)/libwasi_c_sup.a $(bin_dir)/wasi_c_sup.o

wasi_c_sup.o: $(bin_dir) wasi_ext_lib.o wasi-sdk-path
	$(cc) $(CFLAGS) -D_WASI_EMULATED_SIGNAL -Wno-macro-redefined -I . -I $(third_party_dir) -I $(third_party_dir)/wasi_c_sup/include -O -c $(third_party_dir)/wasi_c_sup/wasi_c_sup.c -o $(bin_dir)/wasi_c_sup.o

clean: $(bin_dir)
	rm -rf $(bin_dir)

$(bin_dir):
	mkdir $(bin_dir)

wasi-sdk-path:
ifndef WASI_SDK_PATH
	$(error WASI_SDK_PATH is undefined)
endif
