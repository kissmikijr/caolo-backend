package main

import (
	"context"
	"flag"
	"io"
	"log"
	"net/http"
	"time"

	cao_common "github.com/caolo-game/cao-rt/cao_common_pb"
	"github.com/caolo-game/cao-rt/cao_world_pb"
	cao_world "github.com/caolo-game/cao-rt/cao_world_pb"
	"google.golang.org/grpc"
	"google.golang.org/grpc/backoff"
	"google.golang.org/grpc/connectivity"
)

var addr = flag.String("addr", "localhost:8080", "http service address")
var simAddr = flag.String("simAddr", "localhost:50051", "address of the Simulation Service")

func listenToWorld(conn *grpc.ClientConn, worldState chan *cao_world.RoomEntities) {
	client := cao_world.NewWorldClient(conn)

	for {
		stream, err := client.Entities(context.Background(), &cao_common.Empty{})
		if err != nil {
			panic(err)
		}

		for {
			entitites, err := stream.Recv()
			if err == io.EOF {
				log.Println("Bai")
				return
			}
			if err != nil {
				log.Printf("Error in %v.Entities = %v", client, err)
				break
			}

			worldState <- entitites
		}
		log.Print("Retrying connection")
	}
}

func getRoomData(roomId *cao_common.Axial, client cao_world.WorldClient, send_terrain chan *cao_world_pb.RoomTerrain) {
	terrain, err := client.GetRoomTerrain(context.Background(), roomId)
	if err != nil {
		log.Fatalf("Failed to query terrain of room %v: %v", roomId, err)
	}
	send_terrain <- terrain
}

func initTerrain(conn *grpc.ClientConn, hub *GameStateHub) {
	client := cao_world.NewWorldClient(conn)

	roomList, err := client.GetRoomList(context.Background(), &cao_common.Empty{})
	if err != nil {
		log.Fatalf("Failed to query room list %v", err)
	}

	// Query the room terrains' in parallel
	//

	var ch = make(chan *cao_world_pb.RoomTerrain)
	var todo = len(roomList.Rooms)

	for i := range roomList.Rooms {
		room := roomList.Rooms[i]
		roomId := room.RoomId
		go getRoomData(roomId, client, ch)
	}

	for todo > 0 {
		select {
		case terrain := <-ch:
			roomId := RoomId{
				Q: terrain.RoomId.Q,
				R: terrain.RoomId.R,
			}
			hub.Terrain[roomId] = terrain

			todo -= 1
		}
	}
}

func MinInt(a, b int) int {
	if a < b {
		return a
	}
	return b
}

// Wait until the queen is online
func waitForConnectionReady() *grpc.ClientConn {
	var opts []grpc.DialOption
	opts = append(opts, grpc.WithInsecure(), grpc.WithConnectParams(grpc.ConnectParams{
		Backoff: backoff.Config{
			BaseDelay:  time.Second * 2,
			Multiplier: 1.2,
			Jitter:     0.4,
			MaxDelay:   time.Second * 5,
		},
		MinConnectTimeout: time.Second * 10,
	}))

	conn, err := grpc.Dial(*simAddr, opts...)
	if err != nil {
		log.Fatalf("Failed to connect %v", err)
	}

	var backoff = 500
	for {
		conn.WaitForStateChange(context.Background(), conn.GetState())
		state := conn.GetState()
		switch state {
		case connectivity.Connecting:
			break
		case connectivity.Idle:
		case connectivity.Ready:
			return conn
		case connectivity.Shutdown:
		case connectivity.TransientFailure:
			log.Printf("Connection state changed state=%v. Backing off for %dms", state, backoff)
			conn.Close()

			time.Sleep(time.Duration(backoff) * time.Millisecond)
			backoff = MinInt(backoff*2, 5000)

			conn, err = grpc.Dial(*simAddr, opts...)
			if err != nil {
				log.Fatalf("Failed to connect %v", err)
			}
		}
	}
}

func main() {
	flag.Parse()

	log.Println("Starting")

	conn := waitForConnectionReady()
	defer conn.Close()

	hub := NewGameStateHub()

	go listenToWorld(conn, hub.WorldState)

	go hub.Run()

	initTerrain(conn, hub)

	http.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})

	http.HandleFunc("/object-stream", func(w http.ResponseWriter, r *http.Request) {
		ServeWs(hub, w, r)
	})

	log.Printf("Init done. Listening on %s", *addr)
	log.Fatal(http.ListenAndServe(*addr, nil))
}
