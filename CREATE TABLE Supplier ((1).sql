CREATE TABLE Supplier (
    supplier_id SERIAL PRIMARY KEY,
    name VARCHAR(100),
    city VARCHAR(50)
);

CREATE TABLE Store (
    store_id SERIAL PRIMARY KEY,
    name VARCHAR(100),
    city VARCHAR(50)
);

CREATE TABLE PurchaseOrder (
    order_id SERIAL PRIMARY KEY,
    store_id INT REFERENCES Store(store_id),
    supplier_id INT REFERENCES Supplier(supplier_id),
    order_date DATE,
    expected_delivery DATE,
    actual_delivery DATE
);

CREATE TABLE PurchaseOrderItem (
    item_id SERIAL PRIMARY KEY,
    order_id INT REFERENCES PurchaseOrder(order_id),
    product_id INT,
    quantity INT,
    unit_cost NUMERIC,
    line_total NUMERIC
);

CREATE TABLE DeliveryService (
    delivery_id SERIAL PRIMARY KEY,
    name VARCHAR(50),
    contact_info VARCHAR(100)
);

CREATE TABLE Shipment (
    shipment_id SERIAL PRIMARY KEY,
    order_id INT REFERENCES PurchaseOrder(order_id),
    delivery_id INT REFERENCES DeliveryService(delivery_id),
    ship_date DATE,
    forecast_arrival DATE,
    actual_arrival DATE,
    tracking_number VARCHAR(50),
    shipping_cost NUMERIC,
    status VARCHAR(20)
);

INSERT INTO Supplier (name, city) VALUES
('Kiev Bikes', 'Kiev'),
('Lviv Wheels', 'Lviv');

INSERT INTO Store (name, city) VALUES
('BikeShop Kyiv', 'Kiev'),
('BikeShop Lviv', 'Lviv');

INSERT INTO PurchaseOrder (store_id, supplier_id, order_date, expected_delivery, actual_delivery) VALUES
(1, 1, '2026-01-01', '2026-01-05', '2026-01-06'),
(2, 2, '2026-01-02', '2026-01-07', '2026-01-08');

INSERT INTO PurchaseOrderItem (order_id, product_id, quantity, unit_cost) VALUES
(1, 101, 5, 200),
(1, 102, 3, 150),
(2, 201, 7, 180);

INSERT INTO DeliveryService (name, contact_info) VALUES
('Nova Poshta', '0800-123-456'),
('UkrPoshta', '0800-654-321');

INSERT INTO Shipment (order_id, delivery_id, ship_date, forecast_arrival, actual_arrival, tracking_number, shipping_cost, status) VALUES
(1, 1, '2026-01-01', '2026-01-05', '2026-01-06', 'TRK001', 50, 'Delivered'),
(2, 2, '2026-01-02', '2026-01-07', '2026-01-08', 'TRK002', 60, 'Delivered');


CREATE VIEW SupplierTotals AS
SELECT s.name, SUM(i.line_total) AS total
FROM Supplier s
JOIN PurchaseOrder o ON s.supplier_id = o.supplier_id
JOIN PurchaseOrderItem i ON o.order_id = i.order_id
GROUP BY s.name;

CREATE VIEW LateShipments AS
SELECT shipment_id, order_id, actual_arrival, forecast_arrival
FROM Shipment
WHERE actual_arrival > forecast_arrival;

CREATE VIEW OrdersWithDelivery AS
SELECT o.order_id, o.store_id, o.supplier_id, d.name AS delivery
FROM PurchaseOrder o
JOIN Shipment s ON o.order_id = s.order_id
JOIN DeliveryService d ON s.delivery_id = d.delivery_id;


CREATE FUNCTION total_order(order INT)
RETURNS NUMERIC AS $$
DECLARE total NUMERIC;
BEGIN
    SELECT SUM(line_total) INTO total FROM PurchaseOrderItem WHERE order_id = order;
    RETURN total;
END;
$$ LANGUAGE plpgsql;



CREATE FUNCTION calc_line()
RETURNS TRIGGER AS $$
BEGIN
    NEW.line_total := NEW.quantity * NEW.unit_cost;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_calc_line
BEFORE INSERT OR UPDATE ON PurchaseOrderItem
FOR EACH ROW
EXECUTE FUNCTION calc_line();
