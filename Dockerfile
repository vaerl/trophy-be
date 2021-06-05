FROM fedora:latest
COPY be/target/debug/backend /
CMD /backend
