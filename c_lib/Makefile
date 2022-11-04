bin_dir := bin

all: Archive

Archive: wasi_ext_lib.o json.o
	${AR} rcs $(bin_dir)/libwasi_ext_lib.a $(bin_dir)/wasi_ext_lib.o $(bin_dir)/json.o

wasi_ext_lib.o: $(bin_dir)
	${CC} -O -c wasi_ext_lib.c -o $(bin_dir)/wasi_ext_lib.o

json.o: $(bin_dir)
	${CC} -O -c json/json.c -o $(bin_dir)/json.o

clean: $(bin_dir)
	rm -rf $(bin_dir)

$(bin_dir):
	mkdir $(bin_dir)


