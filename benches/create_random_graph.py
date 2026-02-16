from __future__ import annotations

from dataclasses import dataclass, field
from math import sqrt
import random
import json
from typing import Any
import argparse
import heapq
from collections import deque


# Represent one contact in the contact graph
@dataclass
class Contact:
    # Start time of the contact
    start: float = 0.0

    # End time of the contact
    end: float = 0.0

    # Sending node id
    tx_node: int = 0

    # Receiving node id
    rx_node: int = 0

    # Transmission delay in owlt
    delay: float = 0.0

    # Confidence this contact will happen, currently always set to 1.0 -> full confidence
    confidence: float = 1.0

    # Data rate
    data_rate: int = 100

# Simply coordinates model to create distances in the graph to compute delays
@dataclass
class Coordinates:
    # x-coordinate
    x: int = 0

    # y-coordinate
    y: int = 0

    # z-coordinate
    z: int = 0

    @staticmethod
    def compute_distance(c1: Coordinates, c2: Coordinates) -> float:
        """
        Compute the Euclidean distance between two coordinates.
        :param c1: The first coordinate
        :param c2: The second coordinate
        :return: The Euclidean distance
        """
        return sqrt((c2.x - c1.x)**2 + (c2.y - c2.y)**2 + (c2.z - c2.z)**2)

    @staticmethod
    def create_random_coordinate(max_coordinate: int) -> Coordinates:
        return Coordinates(random.randint(0, max_coordinate), random.randint(0, max_coordinate), random.randint(0, max_coordinate))

@dataclass
class Node:
    # Unique id of the node
    id: int = 0

    # The coordinates for this node. In this simple model, we assume static positions for the node
    coordinates: Coordinates = field(default_factory=Coordinates)

    # Used to store nodes in dicts
    def __hash__(self):
        return self.id

    @staticmethod
    def compute_distance_between_nodes(n1: Node, n2: Node) -> float:
        return Coordinates.compute_distance(n1.coordinates, n2.coordinates)

    def __str__(self) -> str:
        return str(self.id)

@dataclass
class ContactGraph:
    # The list of all nodes in the contact graph
    nodes: list[Node] = field(default_factory=list)

    # Maps from nodes to other nodes. Each mapping a->b contains a list of all contacts between
    # a and b, where a is the tx_node and b the rx_node.
    contacts: dict[Node, dict[Node, list[Contact]]] = field(default_factory=dict)

    # Collect Euclidean distances between all nodes, symmetric dict
    distances: dict[Node, dict[Node, float]] = field(default_factory=dict)


    def create_contacts_for_k_nearest_neighbors(self, node_id: int, k: int, max_time: int):
        """
        Add contacts with k-nearest neighbors to the graph.
        :param node_id: The node who should get some contacts.
        :param k: The number of neighbors to consider for the node.
        :param max_time: The maximum time to be considered.
        :return:
        """
        def add_contact_until_max_is_reached(tx_node_id: int, rx_node_id: int) -> None:
            """
            Add random contact intervals until the max time is reached
            :param tx_node_id: The transmitting node
            :param rx_node_id: The receiver node
            """
            tx_node: Node = self.nodes[tx_node_id]
            rx_node: Node = self.nodes[rx_node_id]

            # Randomly add new contacts until the max_time has been reached
            earliest_time: int = 0
            while earliest_time < max_time:
                start: int = random.randint(earliest_time, max_time)
                distance: int = int(self.distances[tx_node][rx_node]) + 1
                if start + distance > max_time:
                    break
                end: int = random.randint(start + distance, max_time)
                earliest_time = end + 1 + int(0.2 * max_time)

                # Add contact to contact graph, assume symmetric node
                # Always the same data rate and confidence
                contact: Contact = Contact(start, end, tx_node_id, rx_node_id, self.distances[tx_node][rx_node] * 3, 1.0, 100)
                self.add_contact(tx_node, rx_node, contact)

        for neighbor_id, _ in heapq.nsmallest(k, self.distances[self.nodes[node_id]].items(), key=lambda x: x[1])[1:]:
            add_contact_until_max_is_reached(node_id, neighbor_id.id)



    def add_distance(self, node_1: Node, node_2: Node) -> None:
        """
        Adds a distance between the two nodes to the join graph
        :param node_1: The first node
        :param node_2: The second node
        :return:
        """
        distance: float = Node.compute_distance_between_nodes(node_1, node_2)
        if node_1 not in self.distances:
            self.distances[node_1] = dict()
        self.distances[node_1][node_2] = distance
        if node_2 not in self.distances:
            self.distances[node_2] = dict()
        self.distances[node_2][node_1] = distance

    def add_contact(self, tx_node: Node, rx_node: Node, contact: Contact) -> None:
        if tx_node not in self.contacts:
            self.contacts[tx_node] = dict()
        if rx_node not in self.contacts[tx_node]:
            self.contacts[tx_node][rx_node] = []
        self.contacts[tx_node][rx_node].append(contact)

    def translate_into_json_dict(self) -> dict:
        # We have one top-level dict, which stores three subdicts containing vertices, contacts, and distances
        tl_dict: dict[str, Any] = dict()
        vertices_dict: dict[str, list[str]] = dict()
        edges_list: list[Any] = []
        distances_dict: dict[str, dict[str, float]] = dict()
        tl_dict["vertices"] = vertices_dict
        tl_dict["edges"] = edges_list
        tl_dict["distances"] = distances_dict

        def add_nodes_to_vertices_dict(tx_node: Node, rx_node: Node):
            tx_node_id: str = str(tx_node)
            if tx_node_id not in vertices_dict:
                vertices_dict[tx_node_id] = []
            vertices_dict[tx_node_id].append(str(rx_node))

        def add_contacts_to_edges_dict(tx_node: Node, rx_node: Node, contacts: list[Contact]):
            tx_node_id: str = str(tx_node)
            rx_node_id: str = str(rx_node)
            contacts_list: list[list] = []
            for contact in contacts:
                contact_list: list = [tx_node_id, rx_node_id, contact.start, contact.end, [[0.0, contact.confidence, [[contact.start, contact.data_rate, contact.delay]]]]]
                contacts_list.append(contact_list)

            contacts_entry: dict = {"vertices": [tx_node_id, rx_node_id], "contacts": contacts_list}
            edges_list.append(contacts_entry)

        # Go through each contact to build the vertices and contacts dicts
        for tx_node in self.contacts:
            for rx_node in self.contacts[tx_node]:
                add_nodes_to_vertices_dict(tx_node, rx_node)
                add_contacts_to_edges_dict(tx_node, rx_node, self.contacts[tx_node][rx_node])

        # Lastly, create the distances entry
        for tx_node in self.distances:
            tx_node_id: str = str(tx_node)
            for rx_node in self.distances[tx_node]:
                rx_node_id: str = str(rx_node)
                if tx_node_id not in distances_dict:
                    distances_dict[tx_node_id] = dict()
                distances_dict[tx_node_id][rx_node_id] = self.distances[tx_node][rx_node]

        return tl_dict

random.seed(42)

def create_graph_and_convert_to_json(number_of_nodes: int, contacts_in_reach: int, max_coordinate: int, max_time: int) -> dict:
    """
    Creates a random contact graph according to the specification.
    :param number_of_nodes: The number of nodes that should be in the graph
    :param contacts_in_reach: How many nodes a contact will be in contact with. A value of x indicates, that only the x closest neighbors will be in contact.
    :param max_coordinate: The limit of what values a coordinate can reach
    :param max_time: The limit of what time contacts can reach
    """
    contact_graph = ContactGraph()

    # 1. Create all nodes
    for i in range(number_of_nodes):
        contact_graph.nodes.append(Node(i, Coordinates.create_random_coordinate(max_coordinate)))

    # 2. Compute distances between each node
    for i in range(number_of_nodes):
        for j in range(i + 1, number_of_nodes):
            contact_graph.add_distance(contact_graph.nodes[i], contact_graph.nodes[j])

    # 3. Add the closest neighbors to the contacts
    for i in range(number_of_nodes):
        contact_graph.create_contacts_for_k_nearest_neighbors(i, contacts_in_reach, max_coordinate)

    # 4. Translate the contact graph into json and print into a file
    return contact_graph.translate_into_json_dict()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Create a random contact graph")
    parser.add_argument("--no-nodes", type=int, default=100, help="Number of nodes in the graph")
    parser.add_argument("--contact-density", type=int, default=5, help="Density between nodes")
    parser.add_argument("--max-coordinates", type=int, default=1000, help="Max coordinate value")
    parser.add_argument("--max-time", type=int, default=1000, help="Max time for contacts")
    parser.add_argument("--filename", type=str, help="File name for the json file")
    args = parser.parse_args()

    json_data: dict = create_graph_and_convert_to_json(args.no_nodes, args.contact_density, args.max_coordinates, args.max_time)
    with open(args.filename, "w") as f:
        json.dump(json_data, f, indent=1)
