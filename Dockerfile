FROM node:16-alpine as frontend
WORKDIR /frontend
COPY /frontend .
RUN npm i
RUN npm run build

FROM rust:1.98.0
WORKDIR /backend
EXPOSE 8080
COPY /backend .
COPY --from=frontend /frontend/dist dist
RUN cargo install --path .
RUN rm -r target/

CMD ["hwr-ical"]
