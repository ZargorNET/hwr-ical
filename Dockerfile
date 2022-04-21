FROM rustlang/rust:nightly

WORKDIR .
COPY . .
RUN cargo install --path .
RUN rm -r target/

CMD ["hwr-ical"]