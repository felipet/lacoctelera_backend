-- Entities with no references to other entities

-- Ingredient Entity
DROP TABLE IF EXISTS `Ingredient`;
CREATE TABLE `Ingredient` (
  `id` VARCHAR(40),
  `name` varchar(40) NOT NULL,
  `category` ENUM ('spirit', 'bitter', 'soft_drink', 'garnish', 'other') NOT NULL,
  `description` varchar(255),
  CONSTRAINT `Ingredient_PK` PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

-- Table that stores all the supported social profiles.
DROP TABLE IF EXISTS `SocialProfile`;
CREATE TABLE `SocialProfile` (
    `provider_name` varchar(40) NOT NULL,
    `website` VARCHAR(80) NOT NULL,
    CONSTRAINT `SocialProfile_PK` PRIMARY KEY (`provider_name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

-- Author table.
DROP TABLE IF EXISTS `Author`;
CREATE TABLE `Author` (
    `id` VARCHAR(40),
    `name` VARCHAR(40) NOT NULL,
    `surname` VARCHAR(40) NOT NULL,
    `email` VARCHAR(80) NOT NULL,
    `shareable` BOOLEAN DEFAULT FALSE,
    `description` VARCHAR(255),
    `website` VARCHAR(80),
    CONSTRAINT `Author_PK` PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

DROP TABLE IF EXISTS `Tag`;
CREATE TABLE `Tag` (
    `identifier` varchar(40),
    CONSTRAINT `Tag_PK` PRIMARY KEY (`identifier`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

-- Cocktail Entity
DROP TABLE IF EXISTS `Cocktail`;
CREATE TABLE `Cocktail` (
    `id` VARCHAR(40) PRIMARY KEY,
    `name` varchar(40) NOT NULL,
    `description` varchar(255),
    `category` ENUM ('easy', 'medium', 'advanced', 'pro') DEFAULT 'easy',
    `steps` VARCHAR(500) NOT NULL,
    `image_id` VARCHAR(255),
    `url` VARCHAR(255),
    `rating` ENUM ('0', '1', '2', '3', '4', '5') DEFAULT '0',
    `update_date` TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    `creation_date` TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    `owner` VARCHAR(40),
    CONSTRAINT `Cocktail_Owner_FK` FOREIGN KEY (`owner`) REFERENCES `Author`(`id`)
    ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

-- Contains Relation
DROP TABLE IF EXISTS `UsedIngredient`;
CREATE TABLE `UsedIngredient` (
  `cocktail_id` VARCHAR(40),
  `ingredient_id` VARCHAR(40),
  `importance` ENUM ('high', 'med', 'low'),
  `alternatives` VARCHAR(40) DEFAULT NULL,
  `amount` varchar(14) NOT NULL,
  PRIMARY KEY (`cocktail_id`, `ingredient_id`),
  CONSTRAINT `Used_Cocktail_FK` FOREIGN KEY (`cocktail_id`) REFERENCES `Cocktail`(`id`) ON DELETE CASCADE,
  CONSTRAINT `Used_Ingredient_FK` FOREIGN KEY (`ingredient_id`) REFERENCES `Ingredient`(`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

DROP TABLE IF EXISTS `Tagged`;

CREATE TABLE `Tagged` (
    `id` VARCHAR(40) PRIMARY KEY,
    `cocktail_id` VARCHAR(40) NOT NULL,
    `type` ENUM ('author', 'backend') NOT NULL DEFAULT 'backend',
    `tag` varchar(40) NOT NULL,
    CONSTRAINT `Cocktail_ID_FK` FOREIGN KEY (`cocktail_id`) REFERENCES `Cocktail`(`id`),
    CONSTRAINT `Tag_ID_FK` FOREIGN KEY (`tag`) REFERENCES `Tag`(`identifier`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

-- Relation between social profiles and authors.
DROP TABLE IF EXISTS `AuthorHashSocialProfile`;
CREATE TABLE `AuthorHashSocialProfile` (
    `id` VARCHAR(40) NOT NULL,
    `provider_name` VARCHAR(40) NOT NULL,
    `user_name` VARCHAR(40) NOT NULL,
    `author_id` VARCHAR(40) NOT NULL,
    CONSTRAINT `SocialProfile_Provider_FK` FOREIGN KEY (`provider_name`)
        REFERENCES `SocialProfile` (`provider_name`) ON DELETE CASCADE,
    CONSTRAINT `SocialProfile_User_FK` FOREIGN KEY (`author_id`)
        REFERENCES `Author` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
