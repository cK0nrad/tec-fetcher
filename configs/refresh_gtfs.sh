wget https://opendata.tec-wl.be/GTFSV2/TEC-GTFS.zip
rm gtfs/*
unzip -d gtfs/ TEC-GTFS.zip
rm TEC-GTFS.zip
sudo docker compose exec tec-fetcher wget --spider http://localhost:3000/refresh_gtfs?key=GITHUB_KEYREMOVER
sudo docker compose exec tec-gtfs wget --spider http://localhost:3006/refresh_gtfs?key=GITHUB_KEYREMOVER