 docker run    -p 9000:9000    -p 9001:9001    --name minio    -v ~/minio/data:/data    -e "MINIO_ROOT_USER=ROOTNAME"    -e "MINIO_ROOT_PASSWORD=CHANGEME123"    quay.io/minio/minio server /data --console-address ":9001"
vh5kz0fO7BWkIsqc63onDld2YT73WAFjVqYY9JP3
Qi0rJmr47JIkb1f2Iuh2