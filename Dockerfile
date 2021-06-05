FROM fedora:latest
RUN ls
COPY be/target/debug/backend /
CMD /backend
