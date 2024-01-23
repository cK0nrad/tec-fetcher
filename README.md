# OpenMobility - Realtime Processor [Work in Progress]

![OM Logo](./openmobility.png)

**Full Stack Ecosystem for Real-Time Processing and Distribution of Public Transport Data**

## Overview

The OpenMobility Realtime Processor is a core component of the OpenMobility ecosystem, designed for efficient processing and distribution of public transport data. While it's optimized for integration within the OpenMobility stack, it functions effectively as a standalone project. Compatible with standard GTFS and GTFS-RT data formats.

## Featuress

The processor offers a range of features essential for real-time transport data management:

- **Delay Management:** Accurately track and manage delays in transit.
- **Real-Time Positioning:** Monitor the precise location of transit vehicles.
- **Trip Updates:** Receive and process updates related to specific trips.
- **Stop Time Updates:** Stay updated on changes to scheduled stop times.
- **Alerts:** Generate and distribute critical alerts.
- **Remaining Distance Tracking:** Calculate and update the remaining distance for any journey.
- **Historical Data Access:** Access and analyze historical transit data.

## How to Use

To start the server, use the following command in your terminal:

```bash
$ cargo run
```


```bash
#.env
API_URL=http://xxxx:xxxx/gtfs
IP=0.0.0.0
PORT=3000
SECRET=xxxx
```

- `API_URL`: Define the API endpoint.
- `IP`: Specify the IP address to listen on.
- `PORT`: Assign the port for server communication (default is `3000`).
- `SECRET`: Set a secret key for secure operations.

**Note**: The server requires these settings to be explicitly defined. It will not operate with default values and will terminate with an error if they are missing.

## Operational Assumptions

For accurate functioning, the processor assumes that GTFS shape files are as precise as possible. It relies on these files to calculate the remaining distance of a vehicle. Imprecise shape files could lead to inaccurate calculations of remaining distances, delays, and other related features.

<!-- ## OpenMobility Ecosystem

- [OpenMobility UI](https://github.com/cK0nrad/openmobility-ui) 
  - Interface for data visualization

- [OpenMobility Embed Platform](https://github.com/cK0nrad/openmobility-ep)
  -  Platform for integrating OpenMobility data collection

- [OpenMobility Emulator ETS2](https://github.com/cK0nrad/openmobility-ets2)
    - Emulator for ETS2 to test the OpenMobility stack -->
