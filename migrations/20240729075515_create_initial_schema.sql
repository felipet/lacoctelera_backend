-- ----------------------------------------------
-- Initial DB schema for La Coctelera application
-- ----------------------------------------------

-- Cocktail Entity
DROP TABLE IF EXISTS `Cocktail`;
CREATE TABLE `Cocktail` (
  `id` int PRIMARY KEY AUTO_INCREMENT,
  `name` varchar(40) NOT NULL,
  `desc` varchar(255),
  `recipe` varchar(255)
);

-- Ingredient Entity
DROP TABLE IF EXISTS `Ingredient`;
CREATE TABLE `Ingredient` (
  `id` int PRIMARY KEY AUTO_INCREMENT,
  `name` varchar(40) NOT NULL,
  `category` ENUM ('spirit', 'bitter', 'soft_drink', 'garnish', 'juice', 'other') NOT NULL,
  `desc` varchar(255)
);

-- Contains Relation
DROP TABLE IF EXISTS `Contains`;
CREATE TABLE `Contains` (
  `cocktail_id` int,
  `ingredient_id` int,
  `importance` ENUM ('high', 'med', 'low'),
  `alternatives` int,
  `amount` varchar(14) NOT NULL DEFAULT 1,
  PRIMARY KEY (`cocktail_id`, `ingredient_id`)
);

ALTER TABLE `Contains` ADD FOREIGN KEY (`cocktail_id`) REFERENCES `Cocktail` (`id`);

ALTER TABLE `Contains` ADD FOREIGN KEY (`ingredient_id`) REFERENCES `Ingredient` (`id`);

ALTER TABLE `Contains` ADD FOREIGN KEY (`alternatives`) REFERENCES `Ingredient` (`id`);
