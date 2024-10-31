-- -------------------------------------
-- DB Schema for the API author endpoint
-- -------------------------------------

-- Table that stores all the supported social profiles.
CREATE TABLE `SocialProfile` (
    `provider_name` varchar(40) NOT NULL,
    `website` VARCHAR(80) NOT NULL,
    CONSTRAINT `SocialProfile_PK` PRIMARY KEY (`provider_name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

-- Load some Social Media profiles into the DB.
INSERT INTO `SocialProfile` VALUES
    ('Facebook', 'https://www.facebook.com/'),
    ('X', 'https://x.com/'),
    ('Instagram', 'https://www.instagram.com/');

-- Author table.
CREATE TABLE `Author` (
    `id` VARCHAR(40) NOT NULL,
    `name` VARCHAR(40) NOT NULL,
    `surname` VARCHAR(40) NOT NULL,
    `email` VARCHAR(80) NOT NULL,
    `shareable` BOOLEAN DEFAULT TRUE,
    `description` VARCHAR(255),
    `website` VARCHAR(80),
    CONSTRAINT PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

-- Relation between social profiles and authors.
CREATE TABLE `AuthorHashSocialProfile` (
    `id` INT PRIMARY KEY AUTO_INCREMENT,
    `provider_name` VARCHAR(40) NOT NULL,
    `user_name` VARCHAR(40) NOT NULL,
    `author_id` VARCHAR(40) NOT NULL,
    CONSTRAINT `SocialProfile_Provider_FK` FOREIGN KEY (`provider_name`)
        REFERENCES `SocialProfile` (`provider_name`) ON DELETE CASCADE,
    CONSTRAINT `SocialProfile_User_FK` FOREIGN KEY (`author_id`)
        REFERENCES `Author` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;
