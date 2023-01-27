FROM node:16-alpine as frontend
WORKDIR /frontend
COPY /frontend .
RUN npm i
RUN npm run build

FROM rustlang/rust:nightly
WORKDIR /backend
EXPOSE 8080
COPY /backend .
COPY --from=frontend /backend/dist dist
RUN cargo install --path .
RUN rm -r target/

CMD ["hwr-ical"]
