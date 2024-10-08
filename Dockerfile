FROM rust:1-alpine3.20

RUN apk update && apk upgrade && apk add --no-cache \
  gcc musl-dev rust cargo pkgconfig cmake meson \
  zlib-dev libpng-dev jpeg-dev tiff-dev cairo-dev gdk-pixbuf-dev libxml2-dev \
  sqlite-dev glib-dev openjpeg-dev git libc-dev python3-dev

RUN mkdir -p /usr/src/app
WORKDIR /usr/src/app

RUN git clone https://github.com/openslide/openslide.git
WORKDIR /usr/src/app/openslide
RUN mkdir build
RUN meson setup build
RUN meson compile -C build
RUN meson install -C build

WORKDIR /usr/lib

# Note the rustflags option is essential in order for the compiler to
# also link to dynamic libraries (such as openslide)
RUN RUSTFLAGS="-Ctarget-feature=-crt-static" cargo install pamly --features convert
